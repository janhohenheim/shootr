#!/bin/bash

echo "Compiling release binaries"
(cd core/ && cargo build --release);

echo "stopping services..."
ssh jnf service shootr stop;
ssh jnf service pixi stop;

SHOOTR_LOCATION=jnf:/usr/local/src/pixi;
echo "copying files to $SHOOTR_LOCATION..."
scp target/release/shootr $SHOOTR_LOCATION;
scp -r web/client/public web/app.js web/server/yarn.lock web/server/package.json web/server/package-lock.json $SHOOTR_LOCATION;
scp .env $SHOOTR_LOCATION;

echo "Updating npm depencies..."
ssh jnf "(cd /usr/local/src/pixi && yarn install)";

echo "starting services..."
ssh jnf service shootr start;
ssh jnf service pixi start;

echo "Done!"
