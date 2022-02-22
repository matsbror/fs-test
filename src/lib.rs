use std::{str, borrow::Borrow};
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

        match req.method.as_ref() {
            "GET" => self.handle_get(ctx, req).await,
            "POST" => self.handle_post(ctx, req).await,
            "PUT" => self.handle_put(ctx, req).await ,
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
    async fn receive_chunk(&self, ctx: &Context, arg: &Chunk) -> RpcResult<ChunkResponse> {

        info!("receive_chunk called");

        Ok(ChunkResponse {cancel_download: false})
    }

}

impl FsTestActor {

    async fn handle_get(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {

        info!("GET request");

        info!("path: {:?}", req.path);
        info!("queries: {:?}", req.query_string);
        info!("body size: {}", req.body.len());

        Ok(HttpResponse {
            body: "Success!".to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        })
    }

    async fn handle_post(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {

        info!("POST request");

        info!("path: {:?}", req.path);
        info!("queries: {:?}", req.query_string);
        info!("body size: {}", req.body.len());

        let query_map = parse_query_string(&req.query_string);

        let container_name = if query_map.contains_key("container") {
            query_map["container"].clone()
        } else {
            "container".to_string()
        };

        let file_name = if query_map.contains_key("name") {
            query_map["name"].clone()
        } else {
            "file.txt".to_string()
        };

        let bs_client = BlobstoreSender::new();

        // create the container
        let mut resp = bs_client.create_container(ctx, &container_name).await;

        if let Err(e) = resp {
            return Ok(HttpResponse {
                body: json!({ "error": e }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            });
        }

        // let id = ObjectContId {container_id: container_name, object_id: file_name};
        // resp = bs_client.start_upload(ctx, &id).await?;

        // if !resp.success {
        //     return Ok(HttpResponse {
        //         body: json!({ "error": resp.error }).to_string().into_bytes(),
        //         status_code: 400,
        //         ..Default::default()
        //     });
        // }

        // let c_size = 50;
        // let chunks = req.body.chunks(c_size);

        // info!("Number of chunks: {}", chunks.len());

        // let mut sequence_number = 0;
        // for chunk_body in chunks {
        //     let chunk = Chunk {
        //         ids: id.clone(),
        //         bytes: chunk_body.to_vec().clone(),
        //         chunk_size: c_size as u64,
        //         context: None,
        //         sequence_no: sequence_number,
        //         total_bytes: req.body.len() as u64,
        //     };

        //     info!("Send file chunk: {} for {}/{}, sixe {}", chunk.sequence_no, chunk.ids.container_id, chunk.ids.object_id, chunk.bytes.len());
            
        //     resp = bs_client.upload_chunk(ctx, &chunk).await?;

        //     if !resp.success {
        //         return Ok(HttpResponse {
        //             body: json!({ "error": resp.error }).to_string().into_bytes(),
        //             status_code: 400,
        //             ..Default::default()
        //         });
        //     }

        //     sequence_number += 1;
        // }


        // if !resp.success {
        //     return Ok(HttpResponse {
        //         body: json!({ "error": resp.error }).to_string().into_bytes(),
        //         status_code: 400,
        //         ..Default::default()
        //     });
        // }

        Ok(HttpResponse {
            body: "Success!".to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        })
    }

    async fn handle_put(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {

        info!("PUT request");

        info!("path: {:?}", req.path);
        info!("queries: {:?}", req.query_string);
        info!("body size: {}", req.body.len());

        Ok(HttpResponse {
            body: "Success!".to_string().into_bytes(),
            status_code: 200,
            ..Default::default()
        })
    }
    
}