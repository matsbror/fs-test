#[allow(unused_imports)]
use std::{str, borrow::Borrow};
use std::collections::HashMap;
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
#[services(Actor, HttpServer)]
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
            "PUT" => self.handle_put(ctx, op, &req.body, &query_map).await ,
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
    async fn receive_chunk(&self, _ctx: &Context, _arg: &Chunk) -> RpcResult<ChunkResponse> {

        info!("receive_chunk called");

        Err(RpcError::NotImplemented)
    }

}

impl FsTestActor {

    async fn handle_get(&self, _ctx: &Context, op: &str, query_map: &HashMap<&str, String>) -> RpcResult<HttpResponse> {

        info!("GET request. op: {}, query: {:?}", op, query_map);

        Ok(HttpResponse {
            body: "Success!".to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        })
    }

    async fn handle_post(&self, ctx: &Context, op: &str, body: &Vec<u8>, query_map: &HashMap<&str, String>) -> RpcResult<HttpResponse> {

        info!("POST request. op: {}, query: {:?}", op, query_map);

        match op {
            "create_container" =>   create_container(ctx, query_map).await,
            "upload" =>             upload_file(ctx, body, query_map).await,
            _ =>
                Ok(HttpResponse {
                    body: "Success!".to_string().into_bytes(),
                    status_code: 200,
                    ..Default::default()
                })

        }
    }


    async fn handle_put(&self, _ctx: &Context, op: &str, _body: &Vec<u8>, query_map: &HashMap<&str, String>) -> RpcResult<HttpResponse> {

        info!("PUT request. op: {}, query: {:?}", op, query_map);

        Ok(HttpResponse {
            body: "Success!".to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        })
    }
    
}

async fn create_container(_ctx: &Context, _query_map: &HashMap<&str, String>) -> Result<HttpResponse, RpcError> {
    Ok(HttpResponse {
        body: json!({ "error": "create container not implemented yet"}).to_string().into_bytes(),
        status_code: 400,
        ..Default::default()
    })
}

async fn upload_file(ctx: &Context, body: &Vec<u8>, query_map: &HashMap<&str, String>) -> Result<HttpResponse, RpcError> {
    let container_name = query_map.get("container").cloned().unwrap_or("container".to_string());
    let file_name = query_map.get("name").cloned().unwrap_or("file.txt".to_string());

    let bs_client = BlobstoreSender::new();

    // create the container
    let resp = bs_client.create_container(ctx, &container_name).await;

    if let Err(e) = resp {
        return Ok(HttpResponse {
            body: json!({ "error": e }).to_string().into_bytes(),
            status_code: 400,
            ..Default::default()
        });
    }

    // Send the body of the request in one chunk
    let chunk = Chunk {
        container_id: container_name,
        object_id: file_name,
        bytes: body.clone(),
        offset: 0,
        is_last: true,
    };

    let por = PutObjectRequest {
        chunk,
        ..Default::default()
    };
    let poresp = bs_client.put_object(ctx, &por).await;

    if let Err(e) = poresp {
        return Ok(HttpResponse {
            body: json!({ "error": e }).to_string().into_bytes(),
            status_code: 400,
            ..Default::default()
        });
    }


    Ok(HttpResponse {
        body: "Success!".to_string().into_bytes(),
        status_code: 200,
        ..Default::default()
    })
}
