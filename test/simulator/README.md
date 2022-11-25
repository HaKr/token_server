# Token Server access simulation
Some scripts to interact with a running token server

## simulation.js
    > node src/simulation.js [INSTANCE]

Adds five pieces of metadata to the token server and register their tokens.
Subsequently, each token is used to update it's associating metadata, but only
after waiting up to 10s before sending the request. Again, the renewed tokens 
are registered and then used to perform an update request without metadata. Again
these requests are send after up to 3s. 

As a small check, when instance is "cli" or "A", the first token gets removed
before the update is requested. (So that request will fail)

When the instance is "cli", a dump request is send after all of the three phases
of the simulation.


## simulate
    > sh ./simulate

This shell script starts five instances of the simulation script, each with a 
distinctive instance name. Also requests a dump of the metadata after 1s after 
all five scripts are started and again when all scripts have finished