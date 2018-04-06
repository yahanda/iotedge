// Copyright (c) Microsoft. All rights reserved.

use std::borrow::Cow;
use std::collections::HashMap;

use regex::Regex;
use hyper::{Method, StatusCode};

use super::{Builder, Handler, Recognizer};

#[derive(Debug, PartialEq)]
pub struct Parameters {
    captures: Vec<(Option<String>, String)>,
}

impl Parameters {
    pub fn name(&self, k: &str) -> Option<&str> {
        for capture in &self.captures {
            if let &(Some(ref key), ref val) = capture {
                if key == k {
                    return Some(&val);
                }
            }
        }
        None
    }
}

struct RegexRoute {
    pattern: Regex,
    handler: Box<Handler<Parameters>>,
}

#[derive(Default)]
pub struct RegexRoutesBuilder {
    routes: HashMap<Method, Vec<RegexRoute>>,
}

impl Builder for RegexRoutesBuilder {
    type Recognizer = RegexRecognizer;

    fn route<S, H>(mut self, method: Method, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Handler<<Self::Recognizer as Recognizer>::Parameters>,
    {
        let pattern = normalize_pattern(pattern.as_ref());
        let pattern = Regex::new(&pattern).expect("failed to compile regex");
        let handler = Box::new(handler);
        self.routes
            .entry(method)
            .or_insert(Vec::new())
            .push(RegexRoute { pattern, handler });
        self
    }

    fn finish(self) -> Self::Recognizer {
        RegexRecognizer {
            routes: self.routes,
        }
    }
}

pub struct RegexRecognizer {
    routes: HashMap<Method, Vec<RegexRoute>>,
}

impl Recognizer for RegexRecognizer {
    type Parameters = Parameters;

    fn recognize(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<(&Handler<Self::Parameters>, Self::Parameters), StatusCode> {
        let routes = self.routes.get(method).ok_or(StatusCode::NotFound)?;
        for route in routes {
            if let Some(params) = match_route(&route.pattern, path) {
                return Ok((&*route.handler, params));
            }
        }
        Err(StatusCode::NotFound)
    }
}

fn match_route(re: &Regex, path: &str) -> Option<Parameters> {
    re.captures(path).map(|cap| {
        let mut captures = Vec::with_capacity(cap.len());

        for (i, name) in re.capture_names().enumerate() {
            let val = name.map(|n| cap.name(n).expect("missing name"))
                .unwrap_or_else(|| cap.get(i).expect("missing capture"));
            captures.push((name.map(|s| s.to_owned()), val.as_str().to_owned()));
        }
        Parameters { captures }
    })
}

fn normalize_pattern(pattern: &str) -> Cow<str> {
    let pattern = pattern
        .trim()
        .trim_left_matches("^")
        .trim_right_matches("$")
        .trim_right_matches("/");
    match pattern {
        "" => "^/$".into(),
        s => format!("^{}/?$", s).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_name() {
        let pattern = Regex::new("^/test/(?P<name>[^/]+)$").expect("failed to compile regex");
        let params = match_route(&pattern, "/test/mike").expect("failed to get params");
        assert_eq!("mike", params.name("name").unwrap());
    }

    #[test]
    fn params_no_match() {
        let pattern = Regex::new("^/test/(?P<name>[^/]+)$").expect("failed to compile regex");
        let params = match_route(&pattern, "/different/mike");
        assert_eq!(None, params);
    }

    #[test]
    fn params_missing_param() {
        let pattern = Regex::new("^/test/(?P<name>[^/]+)$").expect("failed to compile regex");
        let params = match_route(&pattern, "/test/mike").expect("failed to get params");
        assert_eq!(None, params.name("wrong-param"));
    }
}