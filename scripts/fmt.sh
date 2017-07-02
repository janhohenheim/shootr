#!/bin/bash
echo 'Formatting rust...'
(cd core/ && cargo fmt -- --write-mode=overwrite);
