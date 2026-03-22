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
    pub async fn build_html_str(
        &self,
        poison: &str,
        link_count: u8,
        link_prefix: &LinkPrefix,
    ) -> String {
        let links: String = (0..link_count)
            // uuid as the link suffix so scrapers never see the same link twice
            .map(|_| Uuid::new_v4())
            .map(|id| format!("<li><a href=\"{link_prefix}{id}\">Code Example {id}</a></li>"))
            .collect();

        format!(
            "{}{poison}{}{links}{}",
            self.start_to_poison, self.poison_to_links, self.links_to_end
        )
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
