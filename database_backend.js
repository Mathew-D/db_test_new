      // Database plugin for miniquad/wasm interop
// Exposes mq_db_query for Rust to call via FFI

let db_query_result_buffer = "";

async function mq_db_query(ptr, len, url_ptr, url_len, token_ptr, token_len) {
    // WASM memory is expected to be available as wasm_memory
    console.log("mq_db_query called with ptr", ptr, "len", len, "url_ptr", url_ptr, "url_len", url_len, "token_ptr", token_ptr, "token_len", token_len);
    try {
        const mem = wasm_memory.buffer;
        const decoder = new TextDecoder();
        const body = decoder.decode(new Uint8Array(mem, ptr, len));
        const url = decoder.decode(new Uint8Array(mem, url_ptr, url_len));
        const token = decoder.decode(new Uint8Array(mem, token_ptr, token_len));

        const resp = await fetch(url + "/v2/pipeline", {
            method: "POST",
            headers: {
                "Authorization": `Bearer ${token}`,
                "Content-Type": "application/json"
            },
            body
        });
        db_query_result_buffer = await resp.text();
        console.log("mq_db_query result", db_query_result_buffer);
    } catch (e) {
        db_query_result_buffer = JSON.stringify({ error: "fetch_failed", message: e && e.message ? e.message : String(e) });
        console.error("mq_db_query fetch error", e);
    }
}

function mq_db_query_result_len() {
    return db_query_result_buffer.length;
}

function mq_db_query_fill_result(ptr) {
    if (!db_query_result_buffer) return;
    const enc = new TextEncoder();
    const bytes = enc.encode(db_query_result_buffer);
    new Uint8Array(wasm_memory.buffer, ptr, bytes.length).set(bytes);
}

function mq_db_query_clear_result() {
    db_query_result_buffer = "";
}

function register_plugin(importObject) {
    if (!importObject.env) importObject.env = {};
    importObject.env.mq_db_query = mq_db_query;
    importObject.env.mq_db_query_result_len = mq_db_query_result_len;
    importObject.env.mq_db_query_fill_result = mq_db_query_fill_result;
    importObject.env.mq_db_query_clear_result = mq_db_query_clear_result;
}

window.register_plugin = window.register_plugin;
miniquad_add_plugin({
    name: "database",
    version: "0.1.0",
    register_plugin
});