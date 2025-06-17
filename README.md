# music-rs

rym won't ever give me an api :(
so fuck it. let's "scrape" it like humans and send it to our own database.
don't ever open your server to the public.

the userscript _only_ cares about albums containing bandcamp links. fuck streaming services.
buy your music at a fair price and use navidrome instead of supporting those idiots.

## structure

the server needs a postgres database. stuff the postgres url containing the connection details into an .env-file, with the key `DATABASE_URL`.
once that's done, install sqlx-cli and run `sqlx database create`, then `sqlx migrate run`.
after that, we can run `cargo run --release`, and the server should be up and ready to receive some data.

the userscript can be installed with violentmonkey, which will activate on weekly chart sites such as https://rateyourmusic.com/charts/daily/top/album/2025.01.01-2025.01.07/

you will need to edit the `SERVER_URL` value in the userscript to point to your own server.

navigating to such a weekly chart, three buttons will appear. the `copy albums` button will send off a request to the server which then saves it.

the server has a few endpoints, GETing `/` shows a whole list of all the albums that have been registered. GETing something like `/date/2025-01-01` only shows albums released that day, and `/genre/emoviolence` only shows albums with the defined genre in its list.

## i just want to run it locally

- ok, if you like docker you may `docker-compose up -d` to set the database up.
- if you don't change the specifications, create an `.env` file with the contents `DATABASE_URL=postgres://music:hunter2@localhost:5432/music`
- as described above, `sqlx database create` and `sqlx migrate run`
- `cargo run`, with `--release` if you desire
- the userscript should work without any changes
- go get some data from rym :)

## i can't get it to work!

tbh this is primarily for my own use, so if you can't figure it out make your own thing i guess