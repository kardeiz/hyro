#![feature(pattern)]
#![feature(specialization)]

extern crate hyper;

use std::str::pattern::{Pattern, SearchStep, Searcher};

#[derive(Copy, Clone, Debug)]
pub struct Uri<'a> { 
    pub path: &'a str,
    pub query: Option<&'a str>
}

#[derive(Debug, Clone)]
pub struct Matcher<'a, T=()> {
    pub uri: Uri<'a>,
    pub cursor: usize,
    pub captures: T
}

pub trait PatternExtensions<'a> {
    fn find_c(self, haystack: &'a str) -> Option<usize>;
    fn complete(self, haystack: &'a str) -> bool;
}

impl<'a, P> PatternExtensions<'a> for P where P: Pattern<'a> {
    default fn find_c(self, haystack: &'a str) -> Option<usize> {

        let mut opt_last: Option<usize> = None;
        let mut searcher = self.into_searcher(haystack);
        loop {
            match searcher.next() {
                SearchStep::Match(_, e) => { opt_last = Some(e); },
                _ => { break; }
            }
        }
        opt_last
    }

    default fn complete(self, haystack: &'a str) -> bool {
        let mut searcher = self.into_searcher(haystack);
        loop {
            match searcher.next() {
                SearchStep::Reject(_, _) => { return false; },
                SearchStep::Done => { break; },
                _ => {}
            }
        }
        true
    }
}

impl<'a> PatternExtensions<'a> for char {

    fn find_c(self, haystack: &'a str) -> Option<usize> {

        let mut opt_last: Option<usize> = None;
        let mut searcher = self.into_searcher(haystack);
        if let SearchStep::Match(0, e) = searcher.next() { 
            opt_last = Some(e);
        }
        opt_last
    }

    fn complete(self, haystack: &'a str) -> bool {
        let mut searcher = self.into_searcher(haystack);
        match searcher.next() { 
            SearchStep::Match(0, e) if e == haystack.len() => true ,
            _ => false
        }
    }
}

impl<'a> PatternExtensions<'a> for &'a str {

    fn find_c(self, haystack: &'a str) -> Option<usize> {

        let mut opt_last: Option<usize> = None;
        let mut searcher = self.into_searcher(haystack);
        if let SearchStep::Match(0, e) = searcher.next() { 
            opt_last = Some(e);
        }
        opt_last
    }

    fn complete(self, haystack: &'a str) -> bool {
        let mut searcher = self.into_searcher(haystack);
        match searcher.next() { 
            SearchStep::Match(0, e) if e == haystack.len() => true ,
            _ => false
        }
    }
}

impl<'a, 'b> PatternExtensions<'a> for &'a &'b str {

    fn find_c(self, haystack: &'a str) -> Option<usize> {

        let mut opt_last: Option<usize> = None;
        let mut searcher = self.into_searcher(haystack);
        if let SearchStep::Match(0, e) = searcher.next() { 
            opt_last = Some(e);
        }
        opt_last
    }

    fn complete(self, haystack: &'a str) -> bool {
        let mut searcher = self.into_searcher(haystack);
        match searcher.next() { 
            SearchStep::Match(0, e) if e == haystack.len() => true ,
            _ => false
        }
    }
}

impl<'a> Matcher<'a, ()> {

    pub fn build(uri: &'a ::hyper::uri::RequestUri) ->  Self {
        let (path, query) = match *uri {            
            ::hyper::uri::RequestUri::AbsolutePath(ref s) => {                
                if let Some(pos) = s.find('?') {
                    (&s[..pos], Some(&s[pos+1..]))
                } else {
                    (&s[..], None)
                }
            },
            ::hyper::uri::RequestUri::AbsoluteUri(ref url) => {
                ( url.path(), url.query() )
            },
            _ => panic!("Unexpected request URI")
        };
        let uri = Uri { path: path, query: query };
        Matcher { uri: uri, cursor: 0, captures: () }
    }
}

macro_rules! impls {
    ($([$cur:ty, $nxt:ty, ($($ex:ident,)*), $cap_ty:ty ]),+) => {

        $(
            impl<'a> Matcher<'a, $cur> {
                
                pub fn chomp<P: Pattern<'a>>(&self, pat: P) -> Option<Self> {
                    if let Some(end) = 
                        PatternExtensions::find_c(pat, &self.uri.path[self.cursor..]) {
                        let out = Matcher {
                            uri: self.uri,
                            cursor: self.cursor + end,
                            captures: self.captures
                        };
                        Some(out)
                    } else {
                        None
                    }
                }

                pub fn complete<P: Pattern<'a>>(&self, pat: P) -> Option<Self> {
                    if PatternExtensions::complete(pat, &self.uri.path[self.cursor..]) {
                        let out = Matcher {
                            uri: self.uri,
                            cursor: self.uri.path.len(),
                            captures: self.captures
                        };
                        Some(out)
                    } else {
                        None
                    }
                    
                }

                pub fn take_while<P: Pattern<'a>>(&self, pat: P) 
                    -> Option<Matcher<'a, $nxt>> {

                    if let Some(end) = 
                        PatternExtensions::find_c(pat, &self.uri.path[self.cursor..]) {
                        
                        let (
                            $($ex, )*
                        ) = self.captures;

                        let caps = ( $($ex,)* (self.cursor, self.cursor + end), );

                        let out = Matcher {
                            uri: self.uri,
                            cursor: self.cursor + end,
                            captures: caps
                        };
                        Some(out)
                    } else {
                        None
                    }
                }

                pub fn take_rest(&self) -> Matcher<'a, $nxt> {

                    let end = self.uri.path.len();

                    let (
                        $($ex, )*
                    ) = self.captures;

                    let caps = ( $($ex,)* (self.cursor, end), );

                    let out = Matcher { 
                        cursor: end, 
                        uri: self.uri,
                        captures: caps
                    };

                    out
                }

                pub fn take_until<P: Pattern<'a>>(&self, pat: P) 
                    -> Matcher<'a, $nxt> {

                    let end = self.uri.path[self.cursor..].find(pat)
                        .unwrap_or_else(|| self.uri.path.len() - self.cursor);

                    let (
                        $($ex, )*
                    ) = self.captures;

                    let caps = ( $($ex,)* (self.cursor, self.cursor + end), );

                    let out = Matcher { 
                        cursor: self.cursor + end, 
                        uri: self.uri,
                        captures: caps
                    };

                    out
                }

                impls!(__cap__ $cur, $nxt, ($($ex,)*), $cap_ty);

            }
        )+
    };

    (__cap__ $cur:ty, $nxt:ty, (), $cap_ty:ty ) => {};

    (__cap__ $cur:ty, $nxt:ty, ($($ex:ident,)+), $cap_ty:ty ) => {

        pub fn captures(&self) -> $cap_ty {
            let (
                $($ex, )+
            ) = self.captures;
            (
                $(&self.uri.path[($ex.0)..($ex.1)],)+
            )
            
        }
    };

}

impls!{
    [(), ((usize, usize),), (), ()]
    ,
    [((usize, usize),), ((usize, usize),(usize, usize)), (a,), (&str,)]
    ,
    [
        ((usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize)), 
        (a,b,),
        (&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize)),
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,),
        (&str,&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,),
        (&str,&str,&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,e,),
        (&str,&str,&str,&str,&str)
    ]
    // ,
    // [
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     (a,b,c,d,e,f,),
    //     (&str,&str,&str,&str,&str,&str)
    // ]
    // ,
    // [
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     (a,b,c,d,e,f,g,),
    //     (&str,&str,&str,&str,&str,&str,&str)
    // ]
    // ,
    // [
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     (a,b,c,d,e,f,g,h,),
    //     (&str,&str,&str,&str,&str,&str,&str,&str)
    // ]
    // ,
    // [
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     (a,b,c,d,e,f,g,h,i,),
    //     (&str,&str,&str,&str,&str,&str,&str,&str,&str)
    // ]
    // ,
    // [
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
    //     (a,b,c,d,e,f,g,h,i,j,),
    //     (&str,&str,&str,&str,&str,&str,&str,&str,&str,&str)
    // ]

}

