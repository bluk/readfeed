//! [Really Simple Syndication][rss] is an XML based web content syndication
//! format.
//!
//! Use [`Iter`] as the starting type for parsing a feed.
//!
//! ## Examples
//!
//! ### RSS
//!
//! ```rust
//! use readfeed::rss::{self, ChannelElem, Elem, ItemElem, RssElem};
//!
//! let input = "
//! <rss>
//!     <channel>
//!         <title>Channel Title</title>
//!         <item>
//!             <title>Item Title 1</title>
//!             <link>https://example.com/1</link>
//!             <description>Item Description 1</description>
//!         </item>
//!     </channel>
//! </rss>
//! ";
//!
//! let mut iter = rss::Iter::new(input);
//!
//! let Some(Elem::Rss(mut rss_iter)) = iter.next() else {
//!     panic!();
//! };
//!
//! let Some(RssElem::Channel(mut channel_iter)) = rss_iter.next() else {
//!     panic!();
//! };
//!
//! if let Some(ChannelElem::Title(title)) = channel_iter.next() {
//!     assert_eq!("Channel Title", title.content());
//! } else {
//!     panic!();
//! }
//!
//! let Some(ChannelElem::Item(mut item_iter)) = channel_iter.next() else {
//!     panic!();
//! };
//!
//! if let Some(ItemElem::Title(title)) = item_iter.next() {
//!     assert_eq!("Item Title 1", title.content());
//! } else {
//!     panic!();
//! }
//! if let Some(ItemElem::Link(link)) = item_iter.next() {
//!     assert_eq!("https://example.com/1", link.content());
//! } else {
//!     panic!();
//! }
//! if let Some(ItemElem::Description(desc)) = item_iter.next() {
//!     assert_eq!("Item Description 1", desc.content());
//! } else {
//!     panic!();
//! }
//! assert_eq!(None, item_iter.next());
//!
//! assert_eq!(None, channel_iter.next());
//! assert_eq!(None, rss_iter.next());
//! assert_eq!(None, iter.next());
//! ```
//!
//! [rss]: https://www.rssboard.org/rss-specification

use maybe_xml::{
    token::{
        self,
        prop::{AttributeValue, Attributes, TagName},
        Token,
    },
    Reader,
};

use crate::{xml, Tag};

macro_rules! content_elem {
    ($name:ident $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name<'a> {
            tag: Tag<'a>,
            content: &'a str,
        }

        impl<'a> $name<'a> {
            #[inline]
            #[must_use]
            pub const fn content(&self) -> &'a str {
                self.content
            }

            #[inline]
            #[must_use]
            pub const fn attributes(&self) -> Option<Attributes<'a>> {
                self.tag.attributes()
            }
        }
    };
    ($name:ident, $($nms:ident),+ $(,)?) => {
        content_elem!($name);

        content_elem!($($nms),+);
    };
}

macro_rules! impl_attr {
    ($x:ident, $fn_name:ident, $name:literal) => {
        impl<'a> $x<'a> {
            #[inline]
            #[must_use]
            pub fn $fn_name(&self) -> Option<AttributeValue<'a>> {
                self.tag.find_attribute($name)
            }
        }
    };
}

macro_rules! impl_iter {
    (with_tag $iter_name:ident, $elem_ty:ident, $fn_name:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $iter_name<'a> {
            tag: Tag<'a>,
            reader: Reader<'a>,
            pos: usize,
        }

        impl_iter!($iter_name, $elem_ty, $fn_name);

        impl<'a> $iter_name<'a> {
            #[inline]
            #[must_use]
            pub const fn attributes(&self) -> Option<Attributes<'a>> {
                self.tag.attributes()
            }
        }
    };
    ($iter_name:ident, $elem_ty:ident, $fn_name:expr) => {
        impl<'a> Iterator for $iter_name<'a> {
            type Item = $elem_ty<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(token) = self.reader.tokenize(&mut self.pos) {
                    match token.ty() {
                        token::Ty::StartTag(tag) => {
                            let name = tag.name();
                            let local_name = name.local().as_str();
                            let namespace = name.namespace_prefix().map(|ns| ns.as_str());

                            let content = xml::collect_bytes_until_end_tag(
                                namespace,
                                local_name,
                                &self.reader,
                                &mut self.pos,
                            );

                            return Some($fn_name(Tag::Start(tag), namespace, local_name, content));
                        }
                        token::Ty::EmptyElementTag(tag) => {
                            let name = tag.name();
                            let local_name = name.local().as_str();
                            let namespace = name.namespace_prefix().map(|ns| ns.as_str());

                            return Some($fn_name(
                                Tag::EmptyElement(tag),
                                namespace,
                                local_name,
                                "",
                            ));
                        }
                        token::Ty::Characters(content) => {
                            if content.content().as_str().trim().is_empty() {
                                continue;
                            }
                        }
                        token::Ty::EndTag(_)
                        | token::Ty::ProcessingInstruction(_)
                        | token::Ty::Declaration(_)
                        | token::Ty::Comment(_)
                        | token::Ty::Cdata(_) => {
                            // skip
                        }
                    }

                    return Some($elem_ty::Raw(token));
                }

                None
            }
        }
    };
}

content_elem!(Unknown);

impl<'a> Unknown<'a> {
    #[inline]
    #[must_use]
    pub fn tag_name(&self) -> TagName<'a> {
        self.tag.tag_name()
    }
}

content_elem!(
    ImageUrl,
    ImageTitle,
    ImageLink,
    ImageWidth,
    ImageHeight,
    ImageDescription,
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageElem<'a> {
    Url(ImageUrl<'a>),
    Title(ImageTitle<'a>),
    Link(ImageLink<'a>),
    Width(ImageWidth<'a>),
    Height(ImageHeight<'a>),
    Description(ImageDescription<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

content_elem!(
    ItemTitle,
    ItemLink,
    ItemDescription,
    ItemPubDate,
    ItemComments,
    ItemAuthor,
    ItemCategory,
    ItemEnclosure,
    ItemGuid,
    ItemSource,
);

impl_attr!(ItemCategory, domain, "domain");

impl_attr!(ItemEnclosure, url, "url");
impl_attr!(ItemEnclosure, len, "length");
impl_attr!(ItemEnclosure, ty, "type");

impl_attr!(ItemGuid, is_perma_link, "isPermaLink");

impl_attr!(ItemSource, url, "url");

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemElem<'a> {
    Title(ItemTitle<'a>),
    Link(ItemLink<'a>),
    Description(ItemDescription<'a>),
    Author(ItemAuthor<'a>),
    Category(ItemCategory<'a>),
    Comments(ItemComments<'a>),
    Enclosure(ItemEnclosure<'a>),
    Guid(ItemGuid<'a>),
    PubDate(ItemPubDate<'a>),
    Source(ItemSource<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

content_elem!(SkipHoursHour);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkipHoursElem<'a> {
    Hour(SkipHoursHour<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

content_elem!(SkipDaysDay);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkipDaysElem<'a> {
    Day(SkipDaysDay<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

content_elem!(
    ChannelTitle,
    ChannelLink,
    ChannelDescription,
    ChannelLanguage,
    ChannelCopyright,
    ChannelManagingEditor,
    ChannelWebmaster,
    ChannelPubDate,
    ChannelLastBuildDate,
    ChannelGenerator,
    ChannelDocs,
    ChannelTtl,
    ChannelRating,
    ChannelCategory,
);

content_elem!(Category);
impl_attr!(Category, domain, "domain");

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChannelElem<'a> {
    Title(ChannelTitle<'a>),
    Link(ChannelLink<'a>),
    Description(ChannelDescription<'a>),
    Language(ChannelLanguage<'a>),
    Copyright(ChannelCopyright<'a>),
    ManagingEditor(ChannelManagingEditor<'a>),
    Webmaster(ChannelWebmaster<'a>),
    PubDate(ChannelPubDate<'a>),
    LastBuildDate(ChannelLastBuildDate<'a>),
    Category(ChannelCategory<'a>),
    Generator(ChannelGenerator<'a>),
    Docs(ChannelDocs<'a>),
    // Cloud,
    Ttl(ChannelTtl<'a>),
    Image(ChannelImageIter<'a>),
    Rating(ChannelRating<'a>),
    SkipHours(ChannelSkipHoursIter<'a>),
    SkipDays(ChannelSkipDaysIter<'a>),
    Item(ChannelItemIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RssElem<'a> {
    Channel(ChannelIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Elem<'a> {
    Rss(RssIter<'a>),
    Unknown(Unknown<'a>),
    Raw(Token<'a>),
}

impl<'a> ImageElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> ImageElem<'a> {
        macro_rules! return_content {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        return_content!("url", ImageUrl, ImageElem::Url);
        return_content!("title", ImageTitle, ImageElem::Title);
        return_content!("link", ImageLink, ImageElem::Link);
        return_content!("width", ImageWidth, ImageElem::Width);
        return_content!("height", ImageHeight, ImageElem::Height);
        return_content!("description", ImageDescription, ImageElem::Description);

        ImageElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> ItemElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> ItemElem<'a> {
        macro_rules! return_content {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        return_content!("title", ItemTitle, ItemElem::Title);
        return_content!("link", ItemLink, ItemElem::Link);
        return_content!("description", ItemDescription, ItemElem::Description);
        return_content!("author", ItemAuthor, ItemElem::Author);
        return_content!("comments", ItemComments, ItemElem::Comments);
        return_content!("pubDate", ItemPubDate, ItemElem::PubDate);

        return_content!("guid", ItemGuid, ItemElem::Guid);
        return_content!("category", ItemCategory, ItemElem::Category);
        return_content!("enclosure", ItemEnclosure, ItemElem::Enclosure);
        return_content!("source", ItemSource, ItemElem::Source);

        ItemElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> SkipHoursElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> SkipHoursElem<'a> {
        macro_rules! return_content {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        return_content!("hour", SkipHoursHour, SkipHoursElem::Hour);

        SkipHoursElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> SkipDaysElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> SkipDaysElem<'a> {
        macro_rules! return_content {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        return_content!("day", SkipDaysDay, SkipDaysElem::Day);

        SkipDaysElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> ChannelElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> ChannelElem<'a> {
        macro_rules! return_content {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty { tag, content });
                }
            };
        }

        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("item", ChannelItemIter, ChannelElem::Item);

        return_content!("title", ChannelTitle, ChannelElem::Title);
        return_content!("link", ChannelLink, ChannelElem::Link);
        return_content!("description", ChannelDescription, ChannelElem::Description);
        return_content!("language", ChannelLanguage, ChannelElem::Language);
        return_content!("copyright", ChannelCopyright, ChannelElem::Copyright);
        return_content!(
            "managingEditor",
            ChannelManagingEditor,
            ChannelElem::ManagingEditor
        );
        return_content!("webMaster", ChannelWebmaster, ChannelElem::Webmaster);
        return_content!("pubDate", ChannelPubDate, ChannelElem::PubDate);
        return_content!(
            "lastBuildDate",
            ChannelLastBuildDate,
            ChannelElem::LastBuildDate
        );
        return_content!("generator", ChannelGenerator, ChannelElem::Generator);
        return_content!("docs", ChannelDocs, ChannelElem::Docs);
        return_content!("ttl", ChannelTtl, ChannelElem::Ttl);
        return_content!("rating", ChannelRating, ChannelElem::Rating);

        return_iter!("image", ChannelImageIter, ChannelElem::Image);
        return_iter!("skipHours", ChannelSkipHoursIter, ChannelElem::SkipHours);
        return_iter!("skipDays", ChannelSkipDaysIter, ChannelElem::SkipDays);

        return_content!("category", ChannelCategory, ChannelElem::Category);

        ChannelElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> RssElem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> RssElem<'a> {
        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("channel", ChannelIter, RssElem::Channel);

        RssElem::Unknown(Unknown { tag, content })
    }
}

impl<'a> Elem<'a> {
    fn new(
        tag: Tag<'a>,
        _namespace: Option<&'a str>,
        local_name: &'a str,
        content: &'a str,
    ) -> Elem<'a> {
        macro_rules! return_iter {
            ($local_name: literal, $inner_ty: ident, $elem_ty: expr) => {
                if local_name.eq_ignore_ascii_case($local_name) {
                    return $elem_ty($inner_ty {
                        tag,
                        reader: Reader::from_str(content),
                        pos: 0,
                    });
                }
            };
        }

        return_iter!("rss", RssIter, Elem::Rss);

        Elem::Unknown(Unknown { tag, content })
    }
}

impl_iter!(with_tag ChannelImageIter, ImageElem, ImageElem::new);
impl_iter!(with_tag ChannelItemIter, ItemElem, ItemElem::new);
impl_iter!(with_tag ChannelSkipHoursIter, SkipHoursElem, SkipHoursElem::new);
impl_iter!(with_tag ChannelSkipDaysIter, SkipDaysElem, SkipDaysElem::new);
impl_iter!(with_tag ChannelIter, ChannelElem, ChannelElem::new);

impl_iter!(with_tag RssIter, RssElem, RssElem::new);
impl_attr!(RssIter, version, "version");

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Iter<'a> {
    reader: Reader<'a>,
    pos: usize,
}

impl<'a> Iter<'a> {
    #[inline]
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            reader: Reader::from_str(input),
            pos: 0,
        }
    }
}

impl_iter!(Iter, Elem, Elem::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_lines)]
    #[test]
    fn eval_rss_1() {
        let input = include_str!("../tests/resources/rss-1.xml");

        let mut iter = Iter::new(input);

        let Some(Elem::Raw(token)) = iter.next() else {
            panic!();
        };
        if let token::Ty::ProcessingInstruction(pi) = token.ty() {
            assert_eq!(
                r#"<?xml version="1.0" encoding="ISO-8859-1" ?>"#,
                pi.as_str()
            );
        } else {
            panic!();
        }

        let Some(Elem::Rss(mut rss_iter)) = iter.next() else {
            panic!();
        };
        assert_eq!(Some("0.91"), rss_iter.version().map(|v| v.as_str()));

        let Some(RssElem::Channel(mut channel_iter)) = rss_iter.next() else {
            panic!();
        };

        if let Some(ChannelElem::Title(title)) = channel_iter.next() {
            assert_eq!("Lorem ipsum dolor sit amet.", title.content());
        } else {
            panic!();
        }
        if let Some(ChannelElem::Link(link)) = channel_iter.next() {
            assert_eq!("https://example.com", link.content());
        } else {
            panic!();
        }
        if let Some(ChannelElem::Description(desc)) = channel_iter.next() {
            assert_eq!("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur condimentum erat nec metus velit.", desc.content());
        } else {
            panic!();
        }
        if let Some(ChannelElem::Language(lang)) = channel_iter.next() {
            assert_eq!("en-us", lang.content());
        } else {
            panic!();
        }
        if let Some(ChannelElem::Copyright(copyright)) = channel_iter.next() {
            assert_eq!("Copyright 2020, Lorem ipsum dolor.", copyright.content());
        } else {
            panic!();
        }
        if let Some(ChannelElem::ManagingEditor(editor)) = channel_iter.next() {
            assert_eq!("editor@example.com", editor.content());
        } else {
            panic!();
        }
        if let Some(ChannelElem::Webmaster(webmaster)) = channel_iter.next() {
            assert_eq!("webmaster@example.com", webmaster.content());
        } else {
            panic!();
        }

        let Some(ChannelElem::Image(mut image_iter)) = channel_iter.next() else {
            panic!();
        };
        if let Some(ImageElem::Title(title)) = image_iter.next() {
            assert_eq!("Lorem ipsum dolor sit.", title.content());
        } else {
            panic!();
        }
        if let Some(ImageElem::Url(url)) = image_iter.next() {
            assert_eq!("https://example.com/image1.png", url.content());
        } else {
            panic!();
        }
        if let Some(ImageElem::Link(link)) = image_iter.next() {
            assert_eq!("https://example.com", link.content());
        } else {
            panic!();
        }
        if let Some(ImageElem::Width(width)) = image_iter.next() {
            assert_eq!("1024", width.content());
        } else {
            panic!();
        }
        if let Some(ImageElem::Height(height)) = image_iter.next() {
            assert_eq!("768", height.content());
        } else {
            panic!();
        }
        if let Some(ImageElem::Description(desc)) = image_iter.next() {
            assert_eq!(
                "Lorem ipsum dolor sit amet, consectetur adipiscing.",
                desc.content()
            );
        } else {
            panic!();
        }
        assert_eq!(None, image_iter.next());

        let Some(ChannelElem::Item(mut item_iter)) = channel_iter.next() else {
            panic!();
        };
        if let Some(ItemElem::Title(title)) = item_iter.next() {
            assert_eq!(
                "In accumsan elit a faucibus fermentum. Suspendisse eget ultricies molestie.",
                title.content()
            );
        } else {
            panic!();
        }
        if let Some(ItemElem::Link(link)) = item_iter.next() {
            assert_eq!("https://example.com/1", link.content());
        } else {
            panic!();
        }
        if let Some(ItemElem::Description(desc)) = item_iter.next() {
            assert_eq!("Phasellus maximus porttitor ullamcorper. Duis pellentesque, diam scelerisque fermentum vehicula, ex quam semper augue, porta malesuada velit arcu nec sapien. Vestibulum consequat erat a ante ultrices viverra. Donec ornare enim eu lectus aliquet hendrerit. Pellentesque a justo vel est pulvinar lacinia id sit amet felis. Aliquam sed venenatis eros, ac efficitur lectus. Nam sollicitudin, orci vitae luctus luctus, magna sem accumsan nibh, nec consequat magna magna a diam. Fusce rhoncus, mauris vitae commodo tristique, ipsum neque imperdiet diam, nec convallis orci nisl eu urna. Donec mauris augue, cursus ut semper eget, consequat id ante. In hac habitasse platea dictumst. Nulla maximus scelerisque urna, tristique tristique nisl efficitur a. Curabitur finibus dui id enim tempus aliquam. Morbi pharetra purus eget laoreet auctor. Aenean non lorem et neque varius commodo eu et sapien. Proin convallis pharetra erat sed consequat.", desc.content());
        } else {
            panic!();
        }
        assert_eq!(None, item_iter.next());

        for _ in 0..4 {
            let item_iter = channel_iter.next();
            if let Some(ChannelElem::Item(mut item_iter)) = item_iter {
                assert!(matches!(item_iter.next(), Some(ItemElem::Title(_))));
                assert!(matches!(item_iter.next(), Some(ItemElem::Link(_))));
                assert!(matches!(item_iter.next(), Some(ItemElem::Description(_))));
            } else {
                panic!()
            }
        }

        assert_eq!(None, channel_iter.next());
        assert_eq!(None, rss_iter.next());
        assert_eq!(None, iter.next());
    }
}
