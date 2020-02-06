# web-refresh

Small command line app that uses the [WebDriver](https://www.w3.org/TR/webdriver/)
API to control a Firefox browser instance to load up a URL and then sits around
waiting for the Unix signal `SIGUSR1` to show up and respond by refreshing the page.

You'll need to download the [geckodriver](https://github.com/mozilla/geckodriver/releases)
program to act as the WebDriver server that automates the browser instance.

This is useful when hacking away on a static website that you want to auto-reload
whenever the source files are changed. In combination with something like
[watchexec](https://crates.io/crates/watchexec) this can be used like so:

In terminal 1 run this:

```shell
web-refresh https://localhost:8080/ ~/tools/geckodriver
```

And in terminal 2, run this:

```shell
watchexec --exts html kill -s SIGUSR1 $(pidof web-refresh)
```