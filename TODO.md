# TODO

* Redesign so doesn't exit on save of non-text link, and for display to be persistent instead of constant in and out, with thread and channel.
* Horizontal scroll to support raw.
* Caching and checking of certs.
* Add the proxy (this won't take much since it's essentially the client for origin pull and the server for front end).

## Possible Future:

* Serve gemtext as HTML in a built in web server, 
* HTML from http/https origin served as gemtext. Not sure on either of these if I want to go there.
* Implementing dynamic requests in the server, either by spawning backend processes to generate content or some type of in app templating.
* Implementing the client side handling of 1X responses and prompting for dynamic content.
* Other backends for server, like maybe https to pull content from S3 or github or similar.
