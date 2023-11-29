//! HTML reader to find feed URLs within the document.

use maybe_xml::{
    token::{
        self,
        prop::{Attributes, TagName},
    },
    Reader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elem<'a> {
    FeedUrl(&'a str),
    BaseUrl(&'a str),
}

#[must_use]
fn is_link_tag_name(name: TagName<'_>) -> bool {
    name.as_str().eq_ignore_ascii_case("link")
}

fn eval_link_tag_attributes<'a>(attributes: Attributes<'a>) -> Option<&'a str> {
    let mut href: Option<&'a str> = None;
    let mut rel: Option<&'a str> = None;
    let mut ty: Option<&'a str> = None;

    for attr in attributes {
        let name = attr.name().as_str();

        if name.eq_ignore_ascii_case("href") {
            if let Some(val) = attr.value() {
                href = Some(val.as_str());
            }
        }

        if name.eq_ignore_ascii_case("rel") {
            if let Some(val) = attr.value() {
                rel = Some(val.as_str());
            }
        }

        if name.eq_ignore_ascii_case("type") {
            if let Some(val) = attr.value() {
                ty = Some(val.as_str());
            }
        }
    }

    let href = href?;

    let is_feed_link = rel
        .map(str::trim)
        .map(|rel| rel.eq_ignore_ascii_case("alternate") || rel.eq_ignore_ascii_case("feed"))
        .unwrap_or_default()
        || ty
            .map(str::trim)
            .map(|ty| {
                ty.eq_ignore_ascii_case("application/rss+xml")
                    || ty.eq_ignore_ascii_case("application/atom+xml")
            })
            .unwrap_or_default();
    is_feed_link.then_some(href)
}

#[must_use]
fn is_base_tag_name(name: TagName<'_>) -> bool {
    name.as_str().eq_ignore_ascii_case("base")
}

fn eval_base_tag_attributes(attributes: Attributes<'_>) -> Option<&str> {
    attributes.into_iter().find_map(|attr| {
        if attr.name().as_str().eq_ignore_ascii_case("href") {
            return attr.value().map(|val| val.as_str());
        }
        None
    })
}

#[derive(Debug)]
pub struct Iter<'a> {
    iter: maybe_xml::IntoIter<'a>,
}

impl<'a> Iter<'a> {
    #[inline]
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            iter: Reader::new(input).into_iter(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Elem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        eval(&mut self.iter)
    }
}

#[must_use]
fn eval<'a>(iter: &mut maybe_xml::IntoIter<'a>) -> Option<Elem<'a>> {
    loop {
        let token = iter.next()?;

        match token.ty() {
            token::Ty::StartTag(tag) => {
                let tag_name = tag.name();
                if is_link_tag_name(tag_name) {
                    if let Some(feed_url) = tag.attributes().and_then(eval_link_tag_attributes) {
                        return Some(Elem::FeedUrl(feed_url));
                    }
                }

                if is_base_tag_name(tag_name) {
                    if let Some(base_url) = tag.attributes().and_then(eval_base_tag_attributes) {
                        return Some(Elem::BaseUrl(base_url));
                    }
                }
            }
            token::Ty::EmptyElementTag(tag) => {
                let tag_name = tag.name();
                if is_link_tag_name(tag_name) {
                    if let Some(feed_url) = tag.attributes().and_then(eval_link_tag_attributes) {
                        return Some(Elem::FeedUrl(feed_url));
                    }
                }

                if is_base_tag_name(tag_name) {
                    if let Some(base_url) = tag.attributes().and_then(eval_base_tag_attributes) {
                        return Some(Elem::BaseUrl(base_url));
                    }
                }
            }
            token::Ty::EndTag(_)
            | token::Ty::Characters(_)
            | token::Ty::ProcessingInstruction(_)
            | token::Ty::Declaration(_)
            | token::Ty::Comment(_)
            | token::Ty::Cdata(_) => {
                // skip
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_html_1() {
        let input: &str = include_str!("../tests/resources/html-1.html");
        let mut iter = Iter::new(input);

        assert_eq!(Some(Elem::FeedUrl("/feed.xml")), iter.next());
        // There are 2 "/feed.xml" links
        assert_eq!(Some(Elem::FeedUrl("/feed.xml")), iter.next());
        assert_eq!(None, iter.next());
    }
}
