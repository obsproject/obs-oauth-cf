# OBS OAuth Cloudflare Worker

An implementation of the server-side component of OAuth for use with OBS Studio.  
Written in Rust for some reason.

(Currently not in production)

Requires the following secrets to be set via wrangler (`wranger secret put <name>`):
* `TWITCH_ID`
* `TWITCH_SECRET`
* `RESTREAM_ID`
* `RESTREAM_SECRET`

**Note:** For third-party use the redirect URLs in `wrangler.toml` must be adjusted accordingly and OBS has to be recompiled with the URLs changed.
