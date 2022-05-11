#[allow(unused_imports)]
use std::{str, borrow::Borrow};
use std::collections::{HashMap, BTreeMap};
#[allow(unused_imports)]
use std::path::{Component, Path};
use wasmbus_rpc::actor::prelude::*;
use serde_json::json;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::info;
use wasmcloud_interface_blobstore::*;
mod query_string;
use query_string::parse_query_string;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer, ChunkReceiver)]
struct FsTestActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for FsTestActor {

    async fn handle_request(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {

        let query_map = parse_query_string(&req.query_string);
        let path_segments = Path::components(Path::new(&req.path));
        let op = path_segments.last().unwrap().as_os_str().to_str().unwrap();

        match req.method.as_ref() {
            "GET" => self.handle_get(ctx, op, &query_map).await,
            "POST" => self.handle_post(ctx, op, &req.body, &query_map).await,
            "DELETE" => self.handle_delete(ctx, op, &req.body, &query_map).await ,
            _ =>  Ok(HttpResponse {
                        body: json!({ "error": "cannot handle method" }).to_string().into_bytes(),
                        status_code: 400,
                        ..Default::default()
                    })
        }
    }
}

#[async_trait]
impl ChunkReceiver for FsTestActor {

    /// Receives a file chunk from a blobstore. This must be called AFTER
    /// the StartUpload operation.
    /// It is recommended to keep chunks under 1MB to not exceed wasm memory allocation
    async fn receive_chunk(&self, _ctx: &Context, chunk: &Chunk) -> RpcResult<ChunkResponse> {

        info!("receive_chunk called: container = {:?}, object = {:?}", chunk.container_id, chunk.object_id);

        info!("Length: {:?}", chunk.bytes.len());

        Ok(ChunkResponse::default())
    }

}

impl FsTestActor {

    async fn handle_get(&self, ctx: &Context, op: &str, query_map: &BTreeMap<String, String>) -> RpcResult<HttpResponse> {
        info!("GET request. op: {}, query: {:?}", op, query_map);

        match op {
            "container_exists" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                container_exists(ctx, &container_name).await
            },
            "object_exists" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                let file_name = query_map.get("name").cloned().unwrap_or("file.txt".to_string());
                object_exists(ctx, &container_name, &file_name).await
            },
            "get_object_info" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                let file_name = query_map.get("name").cloned().unwrap_or("file.txt".to_string());
                get_object_info(ctx, &container_name, &file_name).await
            },
            "get_container_info" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                get_container_info(ctx, &container_name).await
            },
            "list_containers" => {
                list_containers(ctx).await
            },
            "list_objects" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                list_objects(ctx, &container_name).await
            },
            "download" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                let file_name = query_map.get("name").cloned().unwrap_or("file.txt".to_string());
                download(ctx, &container_name, &file_name).await
            },
            "async_dl" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                let file_name = query_map.get("name").cloned().unwrap_or("file.txt".to_string());
                let r = async_download(ctx, &container_name, &file_name).await;
                info!("************** after async download");
                r
            },
            _ =>
                Ok(HttpResponse {
                    body: json!({ "success": false, "error": format!("GET operator {:?} not implemented", op) }).to_string().into_bytes(),
                    status_code: 400,
                    ..Default::default()
                })
        }
    }

    async fn handle_post(&self, ctx: &Context, op: &str, body: &Vec<u8>, query_map: &BTreeMap<String, String>) -> RpcResult<HttpResponse> {

        info!("POST request. op: {}, query: {:?}", op, query_map);

        match op {
            "create_container" =>   {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                create_container(ctx, &container_name).await
            },
            "upload" => {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                let file_name = query_map.get("name").cloned().unwrap_or("file.txt".to_string());

                upload_file(ctx, body, &container_name, &file_name).await
            },
            "sink" => {

                let _ = echo_file(ctx, body).await;

                Ok(HttpResponse {
                    body: json!({ "success": true }).to_string().into_bytes(),
                    status_code: 200,
                    ..Default::default()
                })
            },
            "echo" => {
                echo_file(ctx, body).await
            },
            _ =>
                Ok(HttpResponse {
                    body: json!({ "success": false, "error": format!("POST operator {:?} not implemented", op) }).to_string().into_bytes(),
                    status_code: 400,
                    ..Default::default()
                })
        }
    }


    async fn handle_delete(&self, ctx: &Context, op: &str, _body: &Vec<u8>, query_map: &BTreeMap<String, String>) -> RpcResult<HttpResponse> {

        info!("PUT request. op: {}, query: {:?}", op, query_map);

        match op {
            "remove_containers" =>   {
                let container_ids = query_map.range("container".to_string()..).map(|(_k, v)| v).cloned().collect();
                remove_containers(ctx, &container_ids).await
            },
            "remove_objects" =>   {
                let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
                let object_ids = query_map.range("name".to_string()..).map(|(_k, v)| v).cloned().collect();
                remove_objects(ctx, &container_name, &object_ids).await
            },
            _ =>
                Ok(HttpResponse {
                    body: json!({ "success": false, "error": format!("PUT operator {:?} not implemented", op) }).to_string().into_bytes(),
                    status_code: 400,
                    ..Default::default()
                })
        }
    }

}

async fn container_exists(ctx: &Context, container_name: &String) -> Result<HttpResponse, RpcError> {

    let bs_client = BlobstoreSender::new();

    match bs_client.container_exists(ctx, container_name).await {
        Ok(exists) => Ok(HttpResponse {
            body: json!({ "success": true, "container_exists" : exists}).to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        }),
        Err(e) => {
            return Ok(HttpResponse {
                body: json!({ "success": false, "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            });
        }
    }
}


async fn create_container(ctx: &Context, container_name: &String) -> Result<HttpResponse, RpcError> {

    let bs_client = BlobstoreSender::new();

    match bs_client.create_container(ctx, container_name).await {
        Ok(()) => Ok(HttpResponse {
            body: json!({ "success": true}).to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        }),
        Err(e) => {
            return Ok(HttpResponse {
                body: json!({ "success": false, "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            });
        }
    }
}

async fn upload_file(ctx: &Context, body: &Vec<u8>, container_name: &String, file_name: &String) -> Result<HttpResponse, RpcError> {

    let bs_client = BlobstoreSender::new();

    if !bs_client.container_exists(ctx, container_name).await? {
        return Ok(HttpResponse {
            body: json!({ "success": false, "response": "container does not exist" }).to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        });
    }

    // Send the body of the request in one chunk
    let chunk = Chunk {
        container_id: container_name.clone(),
        object_id: file_name.clone(),
        bytes: body.clone(),
        offset: 0,
        is_last: true,
    };

    let por = PutObjectRequest {
        chunk,
        ..Default::default()
    };

    let poresp = bs_client.put_object(ctx, &por).await;

    match poresp {
        Ok(resp) => Ok(HttpResponse {
            body: json!({ "success": true, "response": resp }).to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        }),
        Err(e) => Ok(HttpResponse {
            body: json!({ "error": e }).to_string().into_bytes(),
            status_code: 400,
            ..Default::default()
        }),
    }
}


async fn echo_file(_ctx: &Context, body: &Vec<u8>) -> Result<HttpResponse, RpcError> {

    Ok(HttpResponse {
        body: body.clone(),
        status_code: 200,
        ..Default::default()
    })

}



async fn get_object_info(ctx: &Context,  container_name: &String, file_name: &String) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let o = ContainerObject {
        container_id: container_name.clone(),
        object_id: file_name.clone(),
    };
    let resp = bs_client.get_object_info(ctx, &o).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!(meta).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}



async fn object_exists(ctx: &Context,  container_name: &String, file_name: &String) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let o = ContainerObject {
        container_id: container_name.clone(),
        object_id: file_name.clone(),
    };
    let resp = bs_client.object_exists(ctx, &o).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!({ "success": true, "object_exists": meta }).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}

async fn get_container_info(ctx: &Context,  container_name: &String) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let resp = bs_client.get_container_info(ctx, container_name).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!(meta).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}

async fn list_containers(ctx: &Context) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let resp = bs_client.list_containers(ctx).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!(meta).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}

async fn list_objects(ctx: &Context,  container_name: &String) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let lo_request = ListObjectsRequest {
        container_id: container_name.clone(),
        start_with: None,
        continuation: None,
        end_with: None,
        end_before: None,
        max_items: None
    };
    let resp = bs_client.list_objects(ctx, &lo_request).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!(meta).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}


async fn remove_containers(ctx: &Context,  containers: &ContainerIds) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let resp = bs_client.remove_containers(ctx, containers).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!(meta).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}

async fn remove_objects(ctx: &Context,  container: &ContainerId, object_names: &Vec<ObjectId>) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let objects = RemoveObjectsRequest {
        container_id: container.clone(),
        objects: object_names.clone(),
    };
    let resp = bs_client.remove_objects(ctx, &objects).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: json!(meta).to_string().into_bytes(),
                status_code: 200,
                ..Default::default()
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}

async fn download(ctx: &Context,  container_name: &String, file_name: &String) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let gor = GetObjectRequest {
        container_id: container_name.clone(),
        object_id: file_name.clone(),
        range_start: Some(0),
        range_end: None,
        async_reply: false,
    };

    info!("Send get_object request: {:?}", gor);

    let resp = bs_client.get_object(ctx, &gor).await;

    match resp {
        Ok(meta) =>
            Ok(HttpResponse {
                body: meta.initial_chunk.unwrap().bytes,
                status_code: 200,
                header: HashMap::from([("Content-Type".to_string(), vec!["application/octet-stream".to_string()])]),
            }),
        Err(e) =>
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}

/// This function will request a download to be made asynchronously
/// The method receive_chunk will be called when the downloaded file is sent back.
async fn async_download(ctx: &Context,  container_name: &String, file_name: &String) -> Result<HttpResponse, RpcError> {
    let bs_client = BlobstoreSender::new();

    let gor = GetObjectRequest {
        container_id: container_name.clone(),
        object_id: file_name.clone(),
        range_start: None,
        range_end: None,
        async_reply: true,
    };

    info!("Send get_object request: {:?}", gor);

    let resp = bs_client.get_object(ctx, &gor).await;

    match resp {
        Ok(meta) => 
            Ok(HttpResponse {
                body: meta.initial_chunk.unwrap().bytes,
                status_code: 200,
                header: HashMap::from([("Content-Type".to_string(), vec!["application/octet-stream".to_string()])]),
            }),
        Err(e) => 
            Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            }),
    }
}
