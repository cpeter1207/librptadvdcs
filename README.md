# librptadvdcs

Portable native 48 kHz DCS receive and transmit for rpt_advanced. The ABI 1
descriptor is `rptadv_dcs_descriptor()` in `librptadvdcs.so.1`; consumers
validate its size, version, and `rptadv.dcs` capability before use.

The receive and transmit implementations and their original tests are moved
from `librptadvradio/src/dcs_receive.rs` and `src/dcs_transmit.rs` at commit
`bfdc71738874188055169f4abb3024dafce44a67`. The sole algorithm change
replaces the radio module path for the fixed 48,000 Hz rate constant. Receive
accepts interleaved stereo f32 and reads the left channel; transmit writes mono
f32. Both preserve the original PCM conversion, correction, polarity, clock,
and tail behavior. Radio timer orchestration remains in `librptadvradio`.

Create independent receive and transmit handles during setup. Prepare a new
transmit code/polarity in a separate handle, then use
`transmit_update_config_from` on the running handle to preserve its word phase,
fractional bit accumulator, and tail oscillator. Destroy handles only after
audio callback use stops. See `include/rptadvdcs/rptadvdcs.h` for buffer and
lifecycle requirements.

`make test` exercises the Rust algorithm and C descriptor. `make
platform-verify` checks staged installation and Debian packages. This release
uses `0.1.0-alpha.1`, SONAME `librptadvdcs.so.1`, and Debian packages
`librptadvdcs1` and `librptadvdcs-dev` at version `0.1.0~alpha1-1`.
