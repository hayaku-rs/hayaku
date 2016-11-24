# hayaku-path

The default router for hayaku. Based on [fasthttprouter]().
Handles wildcard parameters; e.g. `/:test` will allow `/one`, `/two`, etc.

Regex support for parameters is planned, allowing you to specify something like
`/:test[a-z]`.
