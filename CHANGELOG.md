# Change Log #

## 0.7.2 - 2017-04-16

* Update `log` to 0.3.7 and `rustc-serialize` to 0.3.23.

## 0.7.1 - 2016-11-13

* Update `rustc-serialize` to 0.3.21.

## 0.7.0 - 2016-05-15

* API CHANGE: `lookup` takes an `IpAddr` again instead of a `SocketAddr`. We
  previously switched to `SocketAddr` after `IpAddr` had been deprecated, but
  it has since been re-added.
