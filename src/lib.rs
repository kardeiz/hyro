#![feature(pattern)]
#![feature(specialization)]

extern crate hyper;

use std::str::pattern::{Pattern, SearchStep, Searcher};

#[derive(Copy, Clone, Debug)]
struct Parts<'a> { 
    path: &'a str,
    query: Option<&'a str>
}

#[derive(Debug, Clone)]
pub struct Matcher<'a, T=()> {
    parts: Parts<'a>,
    cursor: usize,
    captures: T
}

trait PatternExtensions<'a> {
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

impl<'a, T> Matcher<'a, T> {
    pub fn path(&'a self) -> &'a str { self.parts.path }
    pub fn query(&'a self) -> Option<&'a str> { self.parts.query }
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
        let parts = Parts { path: path, query: query };
        Matcher { parts: parts, cursor: 0, captures: () }
    }
}

macro_rules! impls {
    ($([$cur:ty, $nxt:ty, ($($ex:ident,)*), $cap_ty:ty ]),+) => {

        $(
            impl<'a> Matcher<'a, $cur> {
                
                pub fn chomp<P: Pattern<'a>>(&self, pat: P) -> Option<Self> {
                    let path = &self.parts.path[self.cursor..];

                    if let Some(end) = PatternExtensions::find_c(pat, path) {
                        let out = Matcher {
                            parts: self.parts,
                            cursor: self.cursor + end,
                            captures: self.captures
                        };
                        Some(out)
                    } else {
                        None
                    }
                }

                pub fn complete<P: Pattern<'a>>(&self, pat: P) -> Option<Self> {
                    let path = &self.parts.path[self.cursor..];
                    
                    if PatternExtensions::complete(pat, path) {
                        let out = Matcher {
                            parts: self.parts,
                            cursor: self.parts.path.len(),
                            captures: self.captures
                        };
                        Some(out)
                    } else {
                        None
                    }
                    
                }

                pub fn capture_while<P: Pattern<'a>>(&self, pat: P) 
                    -> Option<Matcher<'a, $nxt>> {
                    let path = &self.parts.path[self.cursor..];
                    
                    if let Some(end) = PatternExtensions::find_c(pat, path) {
                        
                        let end = self.cursor + end;

                        let (
                            $($ex, )*
                        ) = self.captures;

                        let caps = ( $($ex,)* (self.cursor, end), );

                        let out = Matcher {
                            parts: self.parts,
                            cursor: end,
                            captures: caps
                        };
                        Some(out)
                    } else {
                        None
                    }
                }

                pub fn capture_rest(&self) -> Matcher<'a, $nxt> {
                    let end = self.parts.path.len();

                    let (
                        $($ex, )*
                    ) = self.captures;

                    let caps = ( $($ex,)* (self.cursor, end), );

                    let out = Matcher { 
                        parts: self.parts,
                        cursor: end, 
                        captures: caps
                    };

                    out
                }

                pub fn capture_until<P: Pattern<'a>>(&self, pat: P) 
                    -> Matcher<'a, $nxt> {
                    let path = &self.parts.path[self.cursor..];

                    let end = path.find(pat).unwrap_or_else(|| path.len() );

                    let end = self.cursor + end;

                    let (
                        $($ex, )*
                    ) = self.captures;

                    let caps = ( $($ex,)* (self.cursor, end), );

                    let out = Matcher { 
                        parts: self.parts,
                        cursor: end, 
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
                $(&self.parts.path[($ex.0)..($ex.1)],)+
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
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,e,f,),
        (&str,&str,&str,&str,&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,e,f,g,),
        (&str,&str,&str,&str,&str,&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,e,f,g,h,),
        (&str,&str,&str,&str,&str,&str,&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,e,f,g,h,i,),
        (&str,&str,&str,&str,&str,&str,&str,&str,&str)
    ]
    ,
    [
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        ((usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize),(usize, usize)), 
        (a,b,c,d,e,f,g,h,i,j,),
        (&str,&str,&str,&str,&str,&str,&str,&str,&str,&str)
    ]

}

