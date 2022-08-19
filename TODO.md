- [x] Semi-rework of buffers
- [ ] 3d rect buffers
- [ ] Check `core` items have `new` and `new_in` methods
- [ ] `strict` feature
- [ ] buffered queues
- [ ] join streams
- [ ] `image` feature (use ffmpeg?)
- [ ] serde support
- [ ] Scoped events (treat events kinda like JoinHandles) 

## Before release
- [x] Check most important stuff is documented
- [x] Create README file for GitHub repo
- [x] Add badges to README and book
- [x] Fix warnings

## Solution to fixed 'forgotten lifetimes' and 'sized queues' simultaneously
`CommandQueue` will be treated kinda like a [`Scope`](https://doc.rust-lang.org/stable/std/thread/struct.Scope.html).
Enqueuing from a `CommandQueue` returns an `Eventual`.

| Blaze Object | Rust lookalike |
| Eventual     | JoinHandle     |