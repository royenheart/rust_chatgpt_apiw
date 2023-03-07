# xtgptr

xt(chat)gpt(GPT)r(wrapper)

## Use(Not a final implementation)

```rust
let key = "OPENAI_KEY";
let key_value = match env::var(key) {
    Ok(val) => val,
    Err(e) => panic!("couldn't find env {}: {}", key, e),
};
let token = ChatLogin::new(&key_value, None).unwrap();
let mut chat = Body::<String>::default();
chat.add_message(Message::new(Roles::User, format!("Today is?")));
chat.perform(&token);
```

Just for fun. 🎶