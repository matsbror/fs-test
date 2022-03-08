# fs-test Actor

This project implements an actor that can be used to test a blobstore provider's 
functionality. It is a simple web service which acts as a facade to the blobstore 
interface.

The actor implements the following functionality:

- Container exists
- Create container (directory/bucket)
- Get Container Info
- List containers
- Remove containers
- Object exists
- Get object information
- List objects
- Remove objects
- Upload an object (file/blob)
- Download an object

# Usage when running

Assuming the httpserver provider that the actor is linked to runs on `localhost:8080` 
the following examples show how to use the web interface.

## Container exists

Looking for container/directory/bucket `cont1`:

`curl -X GET 'http://some.node:8080/container_exists?container=cont1'`

If the container does not exist, the reply will be:

`{"container_exists":false,"success":true}`

The `success` field only denotes that the call itself was successful.

If it exists, the reply is:

`{"container_exists":true,"success":true}`

## Create container

The following will create a container with name `cont1`:

`curl -X POST 'http://localhost:8080/create_container?container=cont1'`

A successful creation will return a http response code 200 and:

`{"success":true}`

## Get container information

## List containers

## Remove containers

## Object exists

## Get object information

## List objects

## Remove objects

## Upload an object



## Download an object

# Building

- To compile the actor and generate a signed Webassembly module, type `make`.
- To load and start the actor you'll need to have a running OCI-compatible
registry. Check that `REG_URL` setting in Makefile is correct, and run
`make push` and `make start` to push the actor to the registry
and start the actor.
Alternately, you can load and start the actor from the host's web ui.
When prompted for the path, 
select `build/fs_test_s.wasm`.

The actor must be linked with an HttpServer capability 
provider with the contract id `wasmcloud:httpserver`. You can start the
provider (TODO: need registry url and more specific instructions here)

It must also be linked to a blobstore provider. At the time of this writing
there are two providers: one for unix file system and one for s3. 


