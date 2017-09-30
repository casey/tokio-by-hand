standard:
  - instant:
    - Producer: future that instantly produces a random byte
    - Consumer: future that instantly "consumes" a byte

  - delayed:
    - Producer: future that produces a random byte after a 1 second delay
    - Consumes: future consumes a byte after a 1 second delay

  - instant_series:
    - Producer: stream that instantly produces random byte
    - Consumer: sink that instantly consumes bytes

  - delayed_series:
    - Producer: stream that produces a byte every second
    - Consumer: sink that consumes a byte every second

  - buffering:
    - Consumer: sink that can buffer until it's full

extended:
  - as above, but using extended API
