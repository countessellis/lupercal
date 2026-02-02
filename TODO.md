# TODO

- Add help mode with --mode help and --help, as well as h key for displaying help within client.
- Currently, when cert doesn't match cache, client errors and exits, and other failures end, instead display error within client.
- Add setup mode that prompts for which mode to configure, then builds the config file, instead of create on missing.
- Comments in the code.
- While file system and client (for proxy or for single gemini server) backends work, adding the code for http/https backend, which will facilitate using things like S3, github, bitbucket, IPFS, etc as the backend.
- Serve gemtext as HTML in a built in web server (possibly with UI expansion described below).
- HTML from http/https origin served as gemtext. Not sure on either of these if I want to go there.
- Implementing dynamic requests in the server, either by spawning backend processes to generate content or some type of in app templating.
- A mechanism similar to .htaccess to create access controls within a capsule.

## Possible UI expansion:

- Implementing the client side handling of 1X responses and prompting for dynamic content.
- Move tui into a ui enum with gui and web as the other two types.
- Build gui that mirrors tui.
- Build web server display, which could also be used for proxying gemini to web.
- Build server monitoring UI, with logs at the bottom like the client has, but two pains above, left showing front end, right showing backend, front end shows requests coming in, waiting for backend, then response sent, backend shows backend requests and responses. Implement this for all UIs.
