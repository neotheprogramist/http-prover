#!/bin/bash	

git clone --depth=1 -b v2.7.0-rc.3 https://github.com/starkware-libs/cairo.git \
    && mv cairo/corelib/ . \
	&& rm -rf cairo/