mod client;

use std::cell::RefCell;

use client::{PosthogClient, PosthogClientConfig, PosthogEvent};
use extism_pdk::*;

type ClientHandle = u32;

thread_local! {
    static CLIENT_STORE: RefCell<Vec<PosthogClient>> = RefCell::new(vec![])
}

#[plugin_fn]
pub fn create_client(config: PosthogClientConfig) -> FnResult<ClientHandle> {
    let client = PosthogClient::new(config);
    let handle: ClientHandle = CLIENT_STORE.with(|store| {
        let mut mut_store = store.borrow_mut();
        mut_store.push(client);
        mut_store.len() as ClientHandle - 1
    });
    return Ok(handle);
}

#[derive(serde::Deserialize, FromBytes)]
#[encoding(Json)]
struct CaptureInput {
    handle: ClientHandle,
    event: PosthogEvent,
}

#[plugin_fn]
pub fn capture(input: CaptureInput) -> FnResult<()> {
    with_client(input.handle, |client| client.capture(input.event));
    return Ok(());
}

#[derive(serde::Deserialize, FromBytes)]
#[encoding(Json)]
struct FlushInput {
    handle: ClientHandle,
}

#[plugin_fn]
pub fn flush(input: FlushInput) -> FnResult<()> {
    with_client(input.handle, |client| client.flush().expect("flush failed"));
    return Ok(());
}

fn with_client(client_id: ClientHandle, f: impl FnOnce(&mut PosthogClient)) {
    CLIENT_STORE.with(|store| {
        let mut mut_store = store.borrow_mut();
        let client = mut_store
            .get_mut(client_id as usize)
            .expect("Client not found");
        return f(client);
    });
}
