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

`curl -X GET 'http://localhost:8080/container_exists?container=cont1'`

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

You can create containers inside a container:

`curl -X POST 'http://localhost:8080/create_container?container=cont1/cont2'`

The container name here is `cont1/cont2`.

## Get container information

Find information about a container:

`curl -X GET 'http://localhost:8080/get_container_info?container=cont1'`

Result (if container exists):

```json
{
  "containerId": "cont1",
  "createdAt": {
    "nsec": 0,
    "sec": 1646892298
  }
}
```

if not:

```json
{
  "error": {
    "HostError": "io: No such file or directory (os error 2)"
  }
}
```
## List containers

`curl  -X GET "http://localhost:8080/list_containers"`

results in (if these containers have been created):

```json
[
  {
    "containerId": "cont1"
  },
  {
    "containerId": "cont1/cont2"
  }
]
```

## Remove containers

`curl  -X DELETE "http://localhost:8080/remove_containers?container=container&container=cont1/cont2""`

Multiple containers can be specified by repeating the query parameter `container=<name>` as seen above.
If a container has nested container, like `cont1/cont2`, then when `cont1` is removed, also the 
nested containers are removed.   

The return is a list of the containers that the system failed to remove. 

## Object exists

`curl -X GET 'http://localhost:8080/object_exists?container=cont1&name=Cargo.lock'`

returns

`{"object_exists":true,"success":true}`

or 

`{"object_exists":false,"success":true}`

## Get object information

`curl -X GET 'http://localhost:8080/get_object_info?container=cont1&name=Cargo.lock'`

returns, for objects that exist:

```json
{
  "containerId": "cont1",
  "contentLength": 70050,
  "lastModified": {
    "nsec": 0,
    "sec": 1646893981
  },
  "objectId": "Cargo.lock"
}
```

## List objects

`curl -X GET 'http://localhost:8080/list_objects?container=cont1'`

example of return with two objects in the container:

```json
{
  "isLast": true,
  "objects": [
    {
      "containerId": "cont1",
      "contentLength": 70050,
      "lastModified": {
        "nsec": 0,
        "sec": 1646893981
      },
      "objectId": "Cargo.lock"
    },
    {
      "containerId": "cont1",
      "contentLength": 463,
      "lastModified": {
        "nsec": 0,
        "sec": 1646893969
      },
      "objectId": "Cargo.toml"
    }
  ]
}
```

## Remove objects

`curl  -X DELETE "http://localhost:8080/remove_containers?container=cont1&name=Cargo.toml"`

returns a list of objects not possible to delete.

Multiple objects can be specified by repeating the `name=<object name>` query parameter. Like:

`curl  -X DELETE "http://localhost:8080/remove_containers?container=cont1&name=Cargo.toml&name=Cargo.lock"`

## Upload an object

To upload (a potentially binary) file do:

`curl -X POST 'http://localhost:8080/upload?container=cont1&name=Cargo.toml' --data-binary @Cargo.toml`

If succesful it will reply:

`{"response":{},"success":true}`

## Download an object

To download a file, named `file.bin` in container `cont1` do:

```
curl -X GET 'http://localhost:8080/download?container=cont1&name=file.bin' > file.bin
```

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
provider with the contract id `wasmcloud:httpserver`. 

It must also be linked to a blobstore provider with contract id `wasmcloud:blobstore`. 
At the time of this writing there are two providers: one for unix file system and one for s3. 



