# Video encoding

### For Linux and Windows

You can download `tangent-cli` from [releasses](https://github.com/passeride/Neothesia/releases)

### For macOS

To encode video you need to install `rust` and `ffmpeg`.

Then compile the tangent-cli: `make build-recorder`

It will compile tangent-cli, from now on it is used as a command line tool

To encode a test.mid file run `./target/release/tangent-cli ./test.mid`

Video will be outputted to `./out` directory
