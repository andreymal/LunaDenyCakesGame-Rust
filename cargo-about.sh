#!/bin/sh

set -e

cargo about generate about.hbs --all-features > data/THIRDPARTY.html
