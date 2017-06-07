use crowbook_text_processing::clean::{ellipsis, guillemets, quotes};
use pulldown_cmark::{html, Options, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES, Parser};

fn typo(text: &str) -> String {
	String::from(ellipsis(guillemets(quotes(text))))
}

pub fn render(md: &str) -> String {
    lazy_static! {
        static ref OPTS: Options = Options::from_bits_truncate(
            OPTION_ENABLE_FOOTNOTES.bits() |
            OPTION_ENABLE_TABLES.bits()
        );
    }

	let pretty = typo(md);
    let mut rendered = String::with_capacity(pretty.len() * 3/2);
    let parser = Parser::new_ext(&pretty, OPTS.clone());
    html::push_html(&mut rendered, parser);
    rendered
}
