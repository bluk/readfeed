# ReadFeed

ReadFeed is a library to process feeds. It provides pull parsers for common feed
formats such as [RSS][rss] and [Atom][atom].

* [Latest API Documentation][api_docs]

## Examples

### RSS

```rust
use readfeed::rss::{self, Elem, ItemElem};

let input = "
<rss>
    <channel>
        <title>Channel Title</title> 
        <item>
            <title>Item Title 1</title> 
            <link>https://example.com/1</link> 
            <description>Item Description 1</description> 
        </item>
    </channel>
</rss>
";

let mut iter = rss::Iter::new(input);

if let Some(Elem::Title(title)) = iter.next() {
    assert_eq!(Some("Channel Title"), title.content());
} else {
    panic!();
}

if let Some(Elem::Item(mut item_iter)) = iter.next() {
    if let Some(ItemElem::Title(title)) = item_iter.next() {
        assert_eq!(Some("Item Title 1"), title.content());
    } else {
        panic!();
    }
    if let Some(ItemElem::Link(link)) = item_iter.next() {
        assert_eq!(Some("https://example.com/1"), link.content());
    } else {
        panic!();
    }
    if let Some(ItemElem::Description(desc)) = item_iter.next() {
        assert_eq!(Some("Item Description 1"), desc.content());
    } else {
        panic!();
    }
    assert_eq!(None, item_iter.next());
} else {
    panic!();
}

assert_eq!(None, iter.next());
```

### Atom

```rust
use readfeed::atom::{self, Elem, EntryElem};

let input = r#"
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <title>Lorem ipsum dolor sit amet.</title>
    <link href="https://example.com/"/>
    <updated>2021-02-24T09:08:10Z</updated>
    <id>urn:uuid:ba9192e8-9e34-4c23-8445-94b67ba316ee</id>
    <entry>
        <title>Lorem ipsum dolor sit.</title>
        <link href="http://example.com/2021/02/24/hello"/>
        <id>urn:uuid:425ba23c-d283-4580-8a3c-3b67aaa6b373</id>
        <updated>2021-02-24T09:08:10Z</updated>
        <summary>Lorem ipsum dolor sit amet, consectetur adipiscing.</summary>
    </entry>
</feed>
"#;

let mut iter = atom::Iter::new(input);

if let Some(Elem::Title(title)) = iter.next() {
    assert_eq!(Some("Lorem ipsum dolor sit amet."), title.content());
} else {
    panic!();
}

if let Some(Elem::Link(link)) = iter.next() {
    assert_eq!(Some("https://example.com/"), link.href().map(|v| v.as_str()));
} else {
    panic!();
}

if let Some(Elem::Updated(updated)) = iter.next() {
    assert_eq!(Some("2021-02-24T09:08:10Z"), updated.content());
} else {
    panic!();
}

if let Some(Elem::Id(id)) = iter.next() {
    assert_eq!(Some("urn:uuid:ba9192e8-9e34-4c23-8445-94b67ba316ee"), id.content());
} else {
    panic!();
}

if let Some(Elem::Entry(mut entry_iter)) = iter.next() {
    if let Some(EntryElem::Title(title)) = entry_iter.next() {
        assert_eq!(Some("Lorem ipsum dolor sit."), title.content());
    } else {
        panic!();
    }
    if let Some(EntryElem::Link(link)) = entry_iter.next() {
        assert_eq!(Some("http://example.com/2021/02/24/hello"), link.href().map(|v| v.as_str()));
    } else {
        panic!();
    }
    if let Some(EntryElem::Id(id)) = entry_iter.next() {
        assert_eq!(Some("urn:uuid:425ba23c-d283-4580-8a3c-3b67aaa6b373"), id.content());
    } else {
        panic!();
    }
    if let Some(EntryElem::Updated(updated)) = entry_iter.next() {
        assert_eq!(Some("2021-02-24T09:08:10Z"), updated.content());
    } else {
        panic!();
    }
    if let Some(EntryElem::Summary(summary)) = entry_iter.next() {
        assert_eq!(Some("Lorem ipsum dolor sit amet, consectetur adipiscing."), summary.content());
    } else {
        panic!();
    }
    assert_eq!(None, entry_iter.next());
} else {
    panic!();
}

assert_eq!(None, iter.next());
```

## Installation

```sh
cargo add readfeed
```

By default, the `std` feature is enabled.

### Alloc only

If the host environment has an allocator but does not have access to the Rust
`std` library:

```sh
cargo add --no-default-features --features alloc readfeed
```

### No allocator / core only

If the host environment does not have an allocator:

```sh
cargo add --no-default-features readfeed
```

## License

Licensed under either of [Apache License, Version 2.0][LICENSE_APACHE] or [MIT
License][LICENSE_MIT] at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[LICENSE_APACHE]: LICENSE-APACHE
[LICENSE_MIT]: LICENSE-MIT
[api_docs]: https://docs.rs/readfeed/
[rss]: https://www.rssboard.org/rss-specification
[atom]: https://datatracker.ietf.org/doc/html/rfc4287