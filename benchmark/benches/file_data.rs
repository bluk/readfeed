use criterion::{criterion_group, criterion_main, Criterion};

const ATOM_1: &str = include_str!("../../readfeed/tests/resources/atom-1.xml");
const HTML_1: &str = include_str!("../../readfeed/tests/resources/html-1.html");
const OPML_1: &str = include_str!("../../readfeed/tests/resources/opml-1.xml");
const RSS_1: &str = include_str!("../../readfeed/tests/resources/rss-1.xml");

fn atom_iter(input: &str) -> u64 {
    use readfeed::atom::{Elem, EntryElem, FeedElem, Iter};

    let mut count = 0;

    let mut iter = Iter::new(input);

    let _ = iter.next();

    let Some(Elem::Feed(feed_iter)) = iter.next() else {
        unreachable!();
    };

    for elem in feed_iter {
        match elem {
            FeedElem::Entry(entry_iter) => {
                for elem in entry_iter {
                    match elem {
                        EntryElem::Id(_) => {
                            count += 1;
                        }
                        EntryElem::Author(_)
                        | EntryElem::Category(_)
                        | EntryElem::Content(_)
                        | EntryElem::Contributor(_)
                        | EntryElem::Link(_)
                        | EntryElem::Published(_)
                        | EntryElem::Rights(_)
                        | EntryElem::Source(_)
                        | EntryElem::Summary(_)
                        | EntryElem::Title(_)
                        | EntryElem::Updated(_)
                        | EntryElem::Unknown(_)
                        | EntryElem::Raw(_) => {}
                    }
                }
            }
            FeedElem::Author(_)
            | FeedElem::Category(_)
            | FeedElem::Contributor(_)
            | FeedElem::Generator(_)
            | FeedElem::Icon(_)
            | FeedElem::Id(_)
            | FeedElem::Link(_)
            | FeedElem::Logo(_)
            | FeedElem::Rights(_)
            | FeedElem::Subtitle(_)
            | FeedElem::Title(_)
            | FeedElem::Updated(_)
            | FeedElem::Unknown(_)
            | FeedElem::Raw(_) => {}
        }
    }

    count
}

fn html_iter(input: &str) -> u64 {
    use readfeed::html::{Elem, Iter};

    let mut count = 0;

    let iter = Iter::new(input);
    for elem in iter {
        match elem {
            Elem::FeedUrl(_) => {
                count += 1;
            }
            Elem::BaseUrl(_) => {}
        }
    }

    count
}

fn opml_iter(input: &str) -> u64 {
    use readfeed::opml::{BodyElem, Elem, Iter, OpmlElem};

    let mut count = 0;

    let mut iter = Iter::new(input);

    let Some(Elem::Raw(_)) = iter.next() else {
        unreachable!();
    };

    let Some(Elem::Opml(mut opml_iter)) = iter.next() else {
        unreachable!();
    };

    let Some(OpmlElem::Head(_)) = opml_iter.next() else {
        unreachable!();
    };

    let Some(OpmlElem::Body(body_iter)) = opml_iter.next() else {
        unreachable!();
    };

    for elem in body_iter {
        match elem {
            BodyElem::Outline(_) => {
                count += 1;
            }
            BodyElem::Unknown(_) | BodyElem::Raw(_) => {}
        }
    }

    count
}

fn rss_iter(input: &str) -> u64 {
    use readfeed::rss::{ChannelElem, Elem, ItemElem, Iter, RssElem};

    let mut count = 0;

    let mut iter = Iter::new(input);

    let _ = iter.next();

    let Some(Elem::Rss(mut rss_iter)) = iter.next() else {
        unreachable!();
    };

    let Some(RssElem::Channel(channel_iter)) = rss_iter.next() else {
        unreachable!();
    };

    for elem in channel_iter {
        match elem {
            ChannelElem::Item(item_iter) => {
                for elem in item_iter {
                    match elem {
                        ItemElem::Title(_) => {
                            count += 1;
                        }
                        ItemElem::Link(_)
                        | ItemElem::Description(_)
                        | ItemElem::Author(_)
                        | ItemElem::Category(_)
                        | ItemElem::Comments(_)
                        | ItemElem::Enclosure(_)
                        | ItemElem::Guid(_)
                        | ItemElem::PubDate(_)
                        | ItemElem::Source(_)
                        | ItemElem::Unknown(_)
                        | ItemElem::Raw(_) => {}
                    }
                }
            }
            ChannelElem::Title(_)
            | ChannelElem::Link(_)
            | ChannelElem::Description(_)
            | ChannelElem::Language(_)
            | ChannelElem::Copyright(_)
            | ChannelElem::ManagingEditor(_)
            | ChannelElem::Webmaster(_)
            | ChannelElem::PubDate(_)
            | ChannelElem::LastBuildDate(_)
            | ChannelElem::Category(_)
            | ChannelElem::Generator(_)
            | ChannelElem::Docs(_)
            | ChannelElem::Ttl(_)
            | ChannelElem::Image(_)
            | ChannelElem::Rating(_)
            | ChannelElem::SkipHours(_)
            | ChannelElem::SkipDays(_)
            | ChannelElem::Unknown(_)
            | ChannelElem::Raw(_) => {}
        }
    }

    count
}

#[allow(clippy::too_many_lines)]
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("atom_iter", |b| {
        b.iter(|| {
            let count = atom_iter(ATOM_1);
            assert_eq!(1, count);
        });
    });
    c.bench_function("html_iter", |b| {
        b.iter(|| {
            let count = html_iter(HTML_1);
            assert_eq!(2, count);
        });
    });
    c.bench_function("opml_iter", |b| {
        b.iter(|| {
            let count = opml_iter(OPML_1);
            assert_eq!(2, count);
        });
    });
    c.bench_function("rss_iter", |b| {
        b.iter(|| {
            let count = rss_iter(RSS_1);
            assert_eq!(5, count);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
