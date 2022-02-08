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
impl ActorReceive for FsTestActor {

    /// Receives a file chunk from a blobstore. This must be called AFTER
    /// the StartUpload operation.
    /// It is recommended to keep chunks under 1MB to not exceed wasm memory allocation
    async fn receive_chunk(&self, ctx: &Context, arg: &Chunk) -> RpcResult<()> {

        info!("receive_chunk called");



        Ok(())
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

        let chunk_size = if query_map.contains_key("chunk_size") {
            query_map["chunk_size"].parse::<usize>().unwrap()
        } else {
            50
        };

        let bs_client = BlobstoreSender::new();

        // create the container
        let mut resp = bs_client.create_container(ctx, &container_name).await?;

        if !resp.success {
            return Ok(HttpResponse {
                body: json!({ "error": resp.error }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            });
        }

        let id = ObjectMetadata {container_id: container_name, id: file_name, size: req.body.len() as u64};
        resp = bs_client.start_upload(ctx, &id).await?;

        if !resp.success {
            return Ok(HttpResponse {
                body: json!({ "error": resp.error }).to_string().into_bytes(),
                status_code: 400,
                ..Default::default()
            });
        }

        let chunks = req.body.chunks(chunk_size);

        info!("Number of chunks: {}", chunks.len());

        let mut sequence_number = 0;
        for chunk_body in chunks {
            let chunk = Chunk {
                object_data: id.clone(),
                bytes: chunk_body.to_vec().clone(),
                chunk_size: chunk_size as u64,
                sequence_no: sequence_number,
            };

            info!("Send file chunk: {} for {}/{}, size {}", chunk.sequence_no, 
                                                            chunk.object_data.container_id, 
                                                            chunk.object_data.id, 
                                                            chunk.bytes.len());
            
            resp = bs_client.upload_chunk(ctx, &chunk).await?;

            if !resp.success {
                return Ok(HttpResponse {
                    body: json!({ "error": resp.error }).to_string().into_bytes(),
                    status_code: 400,
                    ..Default::default()
                });
            }

            sequence_number += 1;
        }


        if !resp.success {
            return Ok(HttpResponse {
                body: json!({ "error": resp.error }).to_string().into_bytes(),
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