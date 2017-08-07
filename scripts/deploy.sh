#!/bin/bash

echo  -e "\033[33;36m Building release binaries... \033[0m" &&
(cd core/ && cargo build --release) &&
(cd web/client && yarn build) &&

echo  -e "\033[33;36m Stopping services... \033[0m" &&
ssh jnf service shootr stop &&
ssh jnf service pixi stop &&

SHOOTR_LOCATION=jnf:/usr/local/src/pixi &&
echo  -e "\033[33;36m Copying files to $SHOOTR_LOCATION... \033[0m" &&
scp target/release/shootr $SHOOTR_LOCATION &&
# Todo: Find out why we can't terminate the next line in '&&', as it stops the script otherwise
scp -r web/client/public web/app.js web/server/yarn.lock web/server/package.json web/server/package-lock.json $SHOOTR_LOCATION;
scp .env $SHOOTR_LOCATION &&

echo  -e "\033[33;36m Updating npm depencies... \033[0m" &&
ssh jnf "(cd /usr/local/src/pixi && yarn install)" &&

echo  -e "\033[33;36m Starting services... \033[0m" &&
ssh jnf service shootr start &&
ssh jnf service pixi start &&

echo  -e "\033[33;32m Done deploying! \033[0m";
