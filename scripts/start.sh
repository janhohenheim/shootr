#!/bin/bash
(cd core/ && cargo run) &
(cd web/ && yarn start) &
