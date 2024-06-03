# botapi-rs
Autogenerated telegram bot api wrapper.
Generates a full serde-based and async telegram bot api wrapper.

This library aims to go above and beyond just generating a http wrapper,
mapping telegram types and methods more cleanly onto rust design patterns
while maintaining low overhead and avoiding unnecessary clones or allocations.

Methods take advantage of generic paremters, slices, and primative types whenever possible and all api types come with an easy to use builder pattern
based interface.

## Features
- Automatically kept up to date with the latest telegram api
- Minimal fluff and boilerplate
- Full async support with tokio
- Support for both long polling and webhooks
- support for large file uploads and local api server
- Automatically generated documentation
- Automatic handling of telegram's ratelimiting

## Adding botapi to your project
You can add via cargo-edit

```bash
cargo add botapi
```

or edit your Cargo.toml to include

```toml
[dependencies]
botapi = { version = "0.0.40", features = [] }
```

There is one available feature: `rhai`, which enabled rhai scripting
support for all telegram types (see below for more information)
Normal users will not require this.


## Select examples
A few examples of what you can do with this library, this is not complete documentation, it just intended as an introduction.

### Create a new client
The bot api client allows low-cost Clone to allow moving into other scopes.
Clients support optional "auto-wait" feature, allowing for delayed auto retry
when a ratelimiting error with a retry_after parameter is handled.
If you have your own handling method for 429 errors you can disable this feature.

```rust
let client = BotBuilder::new("12345:mytoken")
    .unwrap()
    .auto_wait(true) // Automatically retry after 429 errors
    .build();

let handle = client.clone();

async move {
    // async context, cloned client is moved
    let me = handle.get_me().await.unwrap();
}
```

### Simple reply bot
Reply to all messages with `Not cry!`
```rust
// Setup long polling with default updates
let longpoller = LongPoller::new(&client, None);
let mut upd = longpoller.get_updates().await;
while let Some(update) = upd.next().await {
    if let UpdateExt::Message(message) = update {
        client.build_send_message(message.chat.id, "Not cry!")
        .reply_parameters(
            &ReplyParametersBuilder::new(message.message_id).build(),
        )
        .build()
        .await?;
    }
}
```


### Anti-premium bot
React to chat member joins and ban premium users
```rust
let updates = vec![
        "update_id",
        "message",
        "edited_message",
        "channel_post",
        "edited_channel_post",
        "inline_query",
        "chosen_inline_result",
        "callback_query",
        "shipping_query",
        "pre_checkout_query",
        "poll",
        "poll_answer",
        "my_chat_member",
        "chat_member",
        "chat_join_request",
    ]
    .into_iter()
    .map(|v| v.to_owned())
    .collect();

// Setup long polling with all updates including joins
let longpoller = LongPoller::new(&client, Some(updates));
let mut upd = longpoller.get_updates().await;
while let Some(update) = upd.next().await {
    if let UpdateExt::ChatMember(member) = update {
        TG.client()
            .build_ban_chat_member(member.chat.id, member.from.id)
            .build()
            .await?;

        client.build_send_message(member.chat.id, "Begone, blue star!")
            .build()
            .await?;
    }
}
```

### Setup webhooks
*NOTE:* This requires a reverse proxy that supports TLS. TLS is not supported
by this library.

```rust
Webhook::new(
    &client,
    BotUrl::Host("https://bothook.tsinghua.edu.cn"),
    false,
    ([0, 0, 0, 0], 8080).into(),
    None,
)
.get_updates()
.await?
.for_each_concurrent(
    None,
    |update| async move { log::info!("update {:?}", update); },
)
.await
```

## What is the deal with BoxWrapper\<Unbox\<T\>\>?
Warning: nerd stuff ahead. TL;DR: you can treat BoxWrapper\<Box\<T\>\>
and BoxWrapper\<Unbox\<T\>\> as T for most use cases, including
all botapi functions. If you need a T from a BoxWrapper just
use .into() or pass a reference.
its not a bug and it is required. Trust me!


### Nerd explaination
Because of rust does not allow recursive types (types containing an
instance of themselves) except if there is a form of indirection like
an Rc\<T\> or Box\<T\>, there is an inherent incompatibility between
telegram's json types and rust's own type system.

Since this library is autogenerated, we need to break some cycles with Box\<T\>
in order to have valid types. I decided that minimizing Box'ed fields is
a good idea because it makes copying and allocating telegram types
more efficient, but doing this in an optimal fashion is actually equivilent
to [minimum feedback arc set](https://en.wikipedia.org/wiki/Feedback_arc_set), which is NP-complete! Oops.

The only approximation algorithms that exist for this problem that run in
polynomial time result in the types being Box'ed depending on new telegram
types being added in a pseudo-random fashion which results in breaking API
changes every telegram release. The solution I came up with is making any
json type (that is types that may result in cycles) being generic over
Box\<T\> or Unbox\<T\> (a wrapper that isnt a box). This allows for a stable api.
Both Box\<T\> and Unbox\<T\> implement Deref\<T\> and T always implements
From\<BoxWrapper\<Box\<T\>\> / BoxWrapper\<Unbox\<T\>\>, so the overhead of this
workaround should be minimal from a user's point of view.

## What is the deal with the .noskip() methods?
the T.noskip() method on an api type T constructs a NoSkipT type out of T.
This type is serde-compatible with its original type (meaning you can
serialize a T and deserialize it as a NoSkipT), but it doens't use the
`#[serde(skip_serializing_if = "Option::is_none")]` atribute on any field.

The reason you would need this is if you are using a serializer like rmp_serde
that serializes structs as arrays. Skipping fields when serde uses an array
internally causes errors at the deserialization end. This is an edge case but
its useful to have around.

## Rhai bindings
[Rhai](https://rhai.rs) is a simple embeddable scripting language designed
to allow some interoperability with rust types. As rhai requires some
some boilerplate to be written to allow interacting with certain rust types
like enums, this library also optionally autogenerates rhai boilerplate
for all telegram types. Yes, this needs to be integrated with the botapi
library itself because autogeneration is required.

As this adds considerable binary size it is disabled
by default, gated behind the `rhai` feature. Enabling rhai support
does not affect performance in any way aside from binary size.

### Enabling rhai feature
To enable in `Cargo.toml`
```toml
[dependencies]
botapi = { version = "0.0.40", features = [ "rhai" ] }
```

### Initializing rhai engine
In order to interact with botapi types in a rhai environemnt, each
of the required types must be registered with the rhai engine. This
can be done either with all types or with a single type at a time.

```rust
let mut engine = Engine::new();
// Register all types
botapi::gen_types::rhai_helpers::setup_all_rhai(&mut engine);

// Register just one type (recursively registers all fields)
Message::setup_rhai(&mut engine);

```


### Example usage of rhai bindings
Most fields in botapi types are accessible directly to rhai scripts. The
main exception is enums, which are exposed using an api similar to

Example checking enum fields from a `botapi::gen_types::Message` type
the `m` parameter to the anonymous function is assumed to be `botapi::gen_types::Message`
```rust
|m| if m.from.value.is_premium.value {
  "no premium users allowed"
}  else {
  "nice"
}
```

Example checking if a `botapi::gen_types::Message` has text
```rust
// check if value is a unit
|m| m.text.value == ()

// check if enum_type is None
|m m.text.enum_type == "None"
```

## Building the docs
Documentation is generated automatically alongside the library itself.
Docs are live at [https://docs.rs/botapi](https://docs.rs/botapi),
alternatively you can view the docs offline by running

```
cargo build
cargo doc --open
```

## Additional links
[https://github.com/fmeef/dijkstra_bot](https://github.com/fmeef/dijkstra_bot):
 A modular telegram bot framework using this library.

[https://github.com/PaulSonOfLars/telegram-bot-api-spec](https://github.com/PaulSonOfLars/telegram-bot-api-spec):
 This inspiraction for this project and the source of the API spec. (thanks Paul!)
