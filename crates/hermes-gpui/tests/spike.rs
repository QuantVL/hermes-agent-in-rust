//! Task 1 GO/NO-GO spike.
//!
//! Proves that `gpui = "=0.2.2"` (crates.io) supports, on `ubuntu-latest`:
//!   1. a real GPUI window built with `TestAppContext`,
//!   2. a text render whose content can be asserted,
//!   3. dispatch of a GPUI action, and
//!   4. a tokio `reqwest` GET bridging into GPUI through `tokio::sync::mpsc`
//!      and a `cx.notify()` observation.

use std::{sync::mpsc as std_mpsc, time::Duration};

use gpui::{
    actions, point, px, Action, AvailableSpace, TestAppContext, VisualTestContext, WindowHandle,
};
use hermes_gpui::SpikeView;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

actions!(hermes_spike, [SpikeAction]);

/// Captured by the test thread so the async handoff can be asserted without
/// touching GPUI's single-threaded `App` from a tokio worker.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    action_count: usize,
    received: Option<String>,
}

#[test]
fn gpui_022_window_text_action_and_tokio_bridge() {
    // 1. A real tokio runtime and a local mock HTTP server ------------------
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    let (addr_tx, addr_rx) = std_mpsc::channel();
    runtime.spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock server");
        addr_tx
            .send(listener.local_addr().expect("local addr"))
            .expect("send mock server address");

        loop {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).await.expect("read request");
            let body = "spike-ok";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write response");
        }
    });
    let addr = addr_rx
        .recv_timeout(Duration::from_secs(15))
        .expect("mock server must start");

    // 2. GPUI test context, window, root view and a notify observer --------
    let mut cx = TestAppContext::single();
    let (obs_tx, obs_rx) = std_mpsc::channel::<Observation>();

    let window: WindowHandle<SpikeView> = cx.add_window(|_window, _cx| SpikeView::new("spike"));
    let view_entity = window.root(&mut cx).expect("root view");

    let _notify_subscription = window
        .update(&mut cx, |_view, _window, cx| {
            cx.observe_self(move |view, _cx| {
                let _ = obs_tx.send(Observation {
                    action_count: view.action_count,
                    received: view.received.clone(),
                });
            })
        })
        .expect("install cx.notify() observer");

    // Render a first frame so the window has a dispatch tree.
    cx.refresh().expect("refresh window");
    cx.run_until_parked();

    // 3. A real tokio task performs reqwest GET -> tokio::sync::mpsc --------
    let (net_tx, mut net_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let url = format!("http://{addr}/");
    runtime.spawn(async move {
        let response = reqwest::get(&url).await.expect("reqwest GET");
        let body = response.text().await.expect("response body");
        net_tx.send(body).expect("send body over tokio mpsc");
    });

    // 4. A GPUI task receives from that channel and calls cx.notify() ------
    let _bridge = cx.spawn(move |mut async_cx| async move {
        let body = net_rx.recv().await.expect("tokio bridge channel closed");
        window
            .update(&mut async_cx, |view, _window, cx| {
                view.received = Some(body);
                cx.notify();
            })
            .expect("deliver network result into GPUI view");
    });

    // The bridge legitimately parks on the tokio channel while waiting.
    cx.executor().allow_parking();

    // 5. Dispatch one GPUI action ------------------------------------------
    cx.update(|app| {
        app.on_action(move |_: &SpikeAction, app| {
            // Use the entity handle directly: `window.update` here would be
            // re-entrant with the in-progress action dispatch.
            view_entity
                .update(app, |view, cx| {
                    view.action_count += 1;
                    cx.notify();
                })
                .expect("handle dispatched action");
        });
    });
    window
        .update(&mut cx, |_view, window, cx| {
            window.dispatch_action(SpikeAction.boxed_clone(), cx);
        })
        .expect("dispatch action");
    cx.run_until_parked();

    // 6. Assert the rendered text via GPUI's own TextLayout ----------------
    let mut vcx = VisualTestContext::from_window(*window, &cx);
    let (layout, ()) = vcx.draw(
        point(px(0.0), px(0.0)),
        AvailableSpace::min_size(),
        |_window, _cx| "spike",
    );
    assert_eq!(
        layout.text(),
        "spike",
        "GPUI must lay out and render the text node"
    );

    // 7. Pump GPUI until the tokio -> cx.notify() handoff lands ------------
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let observations = loop {
        cx.run_until_parked();
        let collected: Vec<Observation> = obs_rx.try_iter().collect();
        if collected
            .iter()
            .any(|o| o.received.as_deref() == Some("spike-ok"))
        {
            break collected;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "tokio reqwest result never reached a GPUI cx.notify() callback"
        );
        std::thread::sleep(Duration::from_millis(10));
    };

    // 8. Whole vertical-slice assertions ----------------------------------
    assert_eq!(
        cx.read(|app| app.windows().len()),
        1,
        "a GPUI window must exist"
    );
    assert_eq!(
        window
            .update(&mut cx, |view, _window, _cx| view.text.to_string())
            .expect("read view text"),
        "spike",
        "the window root view must render the spike text"
    );
    assert!(
        observations.iter().any(|o| o.action_count == 1),
        "the dispatched action must call cx.notify() on the view"
    );
    assert!(
        observations
            .iter()
            .any(|o| o.received.as_deref() == Some("spike-ok")),
        "reqwest GET body must arrive via tokio mpsc inside a cx.notify() callback"
    );
}
