//! Simple interface to pybtex to extract bibliography entries.

extern crate cpython;

use cpython::{Python, NoArgs, ObjectProtocol, PyErr};

/// Simple bibliography entry with non-standard fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: String,
    pub title: String,
    pub authors: Option<Vec<String>>,
    pub booktitle: Option<String>,
    pub series: Option<String>,
    pub note: Option<String>,
    pub slides: Option<String>,
    pub pdf: Option<String>,
    pub year: u16,
}

/// Parses a bibtex bibliography via pybtex and extracts an `Entry` for each bibliography entry.
pub fn parse_bibliography(file: &str) -> Result<Vec<Entry>, PyErr> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let db = py.import("pybtex.database")?;
    let plain = py.import("pybtex.style.names.plain")?.call(py, "NameStyle", NoArgs, None)?;
    let fmt = py.import("pybtex.richtext")?.call(py, "Text", NoArgs, None)?;
    let bib_data = db.call(py, "parse_file", (file,), None)?;

    let entries = bib_data
        .getattr(py, "entries")?
        .call_method(py, "values", NoArgs, None)?
        .iter(py)?;

    let mut bib_entries = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let key: String = entry.getattr(py, "key")?.extract(py)?;
        let fields = entry.getattr(py, "fields")?;

        let year: u16 = if let Ok(year) = fields.get_item(py, "year").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)) {
            year.parse::<u16>().unwrap()
        } else {
            continue
        };

        let title = if let Ok(title) = fields.get_item(py, "title").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)) {
            title
        } else {
            continue
        };

        let authors = if let Ok(authors) = entry.getattr(py, "persons")?.get_item(py, "author") {
        // In [73]: for ent in bib_data.entries.values():
        //     ...:     for author in ent.persons['author']:
        //     ...:         print(NameStyle().format(author).format().render_as('plaintext'))
            let mut author_names = Vec::new();
            for author in authors.iter(py)? {
                let author = plain.call_method(py, "format", (author?,), None)?
                    .call_method(py, "format", NoArgs, None)?
                    .call_method(py, "render_as", ("plaintext",), None)?
                    .str(py)?
                    .to_string(py)?
                    .into_owned();
                author_names.push(author);
            };
            Some(author_names)
        } else {
            None
        };

        let booktitle = fields.get_item(py, "booktitle").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)).ok();

        let series = fields.get_item(py, "series").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)).ok();

        let note = fields.get_item(py, "note").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)).ok();
        let slides = fields.get_item(py, "_slides").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)).ok();
        let pdf = fields.get_item(py, "_pdf").and_then(|v| fmt.call_method(py, "from_latex", (v,), None)).and_then(|v| v.str(py)).and_then(|v| v.to_string(py).map(String::from)).ok();

        bib_entries.push(Entry {
            key,
            title,
            authors,
            booktitle,
            series,
            note,
            slides,
            pdf,
            year,
        });
    }

    Ok(bib_entries)
}
