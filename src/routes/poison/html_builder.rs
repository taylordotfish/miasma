use std::fmt::Write;

use async_stream::stream;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::sync::OwnedSemaphorePermit;
use uuid::Uuid;

use crate::config::LinkPrefix;

pub const POSION_PAGE: HtmlBuilder = HtmlBuilder::new(include_str!("index.html"));

pub struct HtmlBuilder {
    start_to_poison: &'static str,
    poison_to_links: &'static str,
    links_to_end: &'static str,
}

impl HtmlBuilder {
    /// Build the HTML string response.
    pub fn build_html_stream(
        &self,
        mut poison: impl Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
        link_count: u8,
        link_prefix: &LinkPrefix,
        permit: OwnedSemaphorePermit,
    ) -> impl Stream<Item = Result<Bytes, reqwest::Error>> {
        stream! {
            let _permit = permit;
            yield Ok(Bytes::from(self.start_to_poison));

            while let Some(chunk) = poison.next().await {
                let Ok(mut chunk) = chunk else {
                    yield chunk;
                    continue;
                };

                loop {
                    let Some((i, esc)) = chunk
                        .iter()
                        .enumerate()
                        .filter_map(|(i, b)| match *b {
                            b'<' => Some((i, &b"&lt;"[..])),
                            b'>' => Some((i, b"&gt;")),
                            b'&' => Some((i, b"&amp;")),
                            _ => None,
                        })
                        .next()
                    else {
                        yield Ok(chunk);
                        break;
                    };

                    let remaining = chunk.split_off(i + 1);
                    chunk.truncate(i);
                    yield Ok(chunk);
                    yield Ok(esc.into());
                    chunk = remaining;
                }
            }

            yield Ok(Bytes::from(self.poison_to_links));

            let mut links = Box::pin(Self::build_links_stream(link_count, link_prefix));
            while let Some(chunk) = links.next().await {
                yield Ok(chunk);
            }

            yield Ok(Bytes::from(self.links_to_end));
        }
    }

    fn build_links_stream(link_count: u8, link_prefix: &LinkPrefix) -> impl Stream<Item = Bytes> {
        stream! {
            for _ in 0..link_count {
                let mut buf = String::with_capacity(128);
                _ = write!(&mut buf, "<li><a href=\"{link_prefix}{id}\">Code Example {id}</a></li>", id = Uuid::new_v4());
                yield Bytes::from(buf.into_bytes());
            }
        }
    }
}
impl HtmlBuilder {
    /// This function is a bit insane but it works and let's us write the template in an html file.
    /// Plus, it does all the stupid stuff at compile time! :D
    const fn new(template: &'static str) -> HtmlBuilder {
        let t_bytes = template.as_bytes();
        let poison_marker = "{POISON}".as_bytes();
        let links_marker = "{LINKS}".as_bytes();

        let poison_ind = HtmlBuilder::get_split_ind(t_bytes, poison_marker);
        let links_ind = HtmlBuilder::get_split_ind(t_bytes, links_marker);

        let (head, rest) = template.split_at(poison_ind);
        let (_, rest) = rest.split_at(poison_marker.len());
        let (mid, rest) = rest.split_at(links_ind - (poison_ind + poison_marker.len()));
        let (_, end) = rest.split_at(links_marker.len());

        HtmlBuilder {
            start_to_poison: head,
            poison_to_links: mid,
            links_to_end: end,
        }
    }

    /// Do a sliding window type thing to find where the marker occurs in the template.
    const fn get_split_ind(template: &[u8], marker: &[u8]) -> usize {
        let mut ind = 0;
        while ind + marker.len() <= template.len() {
            let mut cur = 0;
            while cur < marker.len() {
                if template[ind + cur] != marker[cur] {
                    break;
                }
                cur += 1;
            }
            if cur == marker.len() {
                return ind;
            }
            ind += 1;
        }
        panic!("failed to find marker in template");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_extracts_parts() {
        let template = "start{POISON}middle{LINKS}end";
        let HtmlBuilder {
            start_to_poison,
            poison_to_links,
            links_to_end,
        } = HtmlBuilder::new(template);
        assert_eq!(start_to_poison, "start");
        assert_eq!(poison_to_links, "middle");
        assert_eq!(links_to_end, "end");
    }

    #[test]
    #[should_panic]
    fn new_fails_if_markers_out_of_order() {
        HtmlBuilder::new("can't have {LINKS} before {POISON}!");
    }

    #[test]
    fn get_split_ind_returns_start_of_marker() {
        let template = "foo{HERE}bar";
        let expected = 3;
        let ind = HtmlBuilder::get_split_ind(template.as_bytes(), "{HERE}".as_bytes());
        assert_eq!(ind, expected);
    }

    #[test]
    #[should_panic]
    fn get_split_ind_panics_on_missing_marker() {
        HtmlBuilder::get_split_ind("this doesn't contain".as_bytes(), "THE MARKER".as_bytes());
    }
}
