use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::process::Command;

/// Real Chrome, daemon, CLI and MCP: a slow click must return the daemon's
/// diagnosis rather than lose its reply to a competing transport deadline.
#[test]
#[ignore = "requires Chrome; the real CDP deadline takes 30 seconds"]
fn slow_find_click_keeps_the_daemon_reply() {
    let sockets = tempfile::tempdir().unwrap();
    let call = |tool: &str, args: Value| {
        let output = Command::new(env!("CARGO_BIN_EXE_agent-browser"))
            .env("AGENT_BROWSER_SOCKET_DIR", sockets.path())
            .env_remove("AGENT_BROWSER_NAMESPACE")
            .env_remove("AGENT_BROWSER_CDP")
            .env_remove("AGENT_BROWSER_AUTO_CONNECT")
            .env_remove("AGENT_BROWSER_PROVIDER")
            .args(["mcp", "call", tool, &args.to_string()])
            .output()
            .unwrap();
        serde_json::from_slice::<Value>(&output.stdout)
            .unwrap_or_else(|e| panic!("{e}: {output:?}"))
    };
    let html = r##"<a href="#" onclick="window.clicks=(window.clicks||0)+1; const end=performance.now()+35000; while(performance.now()<end){}; return false">Slow</a>"##;
    let url = format!("data:text/html;base64,{}", STANDARD.encode(html));
    let opened = call(
        "agent_browser_open",
        json!({"session":"slow-reply", "url":url}),
    );
    assert_eq!(opened["isError"], false, "{opened}");

    let clicked = call(
        "agent_browser_find",
        json!({
            "session":"slow-reply", "locator":"role", "value":"link", "name":"Slow"
        }),
    );
    let clicks = call(
        "agent_browser_eval",
        json!({"session":"slow-reply", "script":"window.clicks"}),
    );
    let closed = call("agent_browser_close", json!({"session":"slow-reply"}));

    assert_eq!(clicked["isError"], true, "{clicked}");
    assert_eq!(
        clicked["structuredContent"]["response"]["code"], "page_wedged",
        "{clicked}"
    );
    assert_eq!(
        clicks["structuredContent"]["response"]["data"]["result"], 1,
        "{clicks}"
    );
    assert_eq!(closed["isError"], false, "{closed}");
}
