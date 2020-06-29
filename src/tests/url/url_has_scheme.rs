//  ██████╗  █████╗ ███████╗███████╗██╗███╗   ██╗ ██████╗
//  ██╔══██╗██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝
//  ██████╔╝███████║███████╗███████╗██║██╔██╗ ██║██║  ███╗
//  ██╔═══╝ ██╔══██║╚════██║╚════██║██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║███████║███████║██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝

#[cfg(test)]
mod passing {
    use crate::url;

    #[test]
    fn mailto() {
        assert!(url::url_has_scheme(
            "mailto:somebody@somewhere.com?subject=hello"
        ));
    }

    #[test]
    fn tel() {
        assert!(url::url_has_scheme("tel:5551234567"));
    }

    #[test]
    fn ftp_no_slashes() {
        assert!(url::url_has_scheme("ftp:some-ftp-server.com"));
    }

    #[test]
    fn ftp_with_credentials() {
        assert!(url::url_has_scheme(
            "ftp://user:password@some-ftp-server.com"
        ));
    }

    #[test]
    fn javascript() {
        assert!(url::url_has_scheme("javascript:void(0)"));
    }

    #[test]
    fn http() {
        assert!(url::url_has_scheme("http://news.ycombinator.com"));
    }

    #[test]
    fn https() {
        assert!(url::url_has_scheme("https://github.com"));
    }

    #[test]
    fn mailto_uppercase() {
        assert!(url::url_has_scheme(
            "MAILTO:somebody@somewhere.com?subject=hello"
        ));
    }
}

//  ███████╗ █████╗ ██╗██╗     ██╗███╗   ██╗ ██████╗
//  ██╔════╝██╔══██╗██║██║     ██║████╗  ██║██╔════╝
//  █████╗  ███████║██║██║     ██║██╔██╗ ██║██║  ███╗
//  ██╔══╝  ██╔══██║██║██║     ██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║██║███████╗██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝

#[cfg(test)]
mod failing {
    use crate::utils;

    #[test]
    fn url_with_no_protocol() {
        assert!(!url::url_has_scheme(
            "//some-hostname.com/some-file.html"
        ));
    }

    #[test]
    fn relative_path() {
        assert!(!url::url_has_scheme("some-hostname.com/some-file.html"));
    }

    #[test]
    fn relative_to_root_path() {
        assert!(!url::url_has_scheme("/some-file.html"));
    }

    #[test]
    fn empty_string() {
        assert!(!url::url_has_scheme(""));
    }
}
