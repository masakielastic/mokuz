use hyper::{
  Response,
  body::Bytes,
  body::Frame
};

use hyper_util::server;
use hyper_util::rt::{
  TokioIo, TokioExecutor
};

use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};

use futures_util::{
  stream, stream::Once
};

use rustls::version;
use rustls_pki_types::{
    CertificateDer, PrivateKeyDer, pem::PemObject
};

use std::{
  convert::Infallible, sync::Arc
};

use http_body_util::{
  StreamBody
};

fn load_tls_config() -> Arc<rustls::ServerConfig> {
    let cert_file = std::fs::File::open("cert.pem").expect("cannot open certificate");
    let key_file = std::fs::File::open("key.pem").expect("cannot open private key");

    let certs: Vec<CertificateDer> = CertificateDer::pem_reader_iter(cert_file)
        .collect::<Result<_, _>>()
        .expect("invalid certificate");

    let key: PrivateKeyDer = PrivateKeyDer::pem_reader_iter(key_file)
        .next()
        .expect("no private key found")
        .expect("invalid private key");

    let mut config = rustls::ServerConfig::builder_with_protocol_versions(&[&version::TLS13, &version::TLS12])
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("TLS config invalid");
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
}


async fn handle_request(_req: hyper::Request<hyper::body::Incoming>)
    -> Result<Response<StreamBody<Once<std::future::Ready<Result<Frame<Bytes>, Infallible>>>>>, hyper::Error>
{
    let content = match tokio::fs::read("example.txt").await {
        Ok(bytes) => bytes,
        Err(_) => {
            let response = Response::builder()
                .status(500)
                .body(StreamBody::new(stream::once(std::future::ready(Ok(Frame::data(Bytes::from_static(b"Internal Server Error")))))))
                .unwrap();
            return Ok(response);
        }
    };

    let bytes = Bytes::from(content);
    let frame = Frame::data(bytes);
    let body = StreamBody::new(stream::once(std::future::ready(Ok::<Frame<Bytes>, Infallible>(frame))));

    Ok(Response::new(body))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8443";
    let listener = TcpListener::bind(addr).await.unwrap();
    let config = load_tls_config();
    let acceptor = TlsAcceptor::from(config);

    println!("🚀 Listening on https://0.0.0.0:8443");

    tokio::select! {
        _ = async {
            loop {
                let (stream, _) = listener.accept().await?;
                let acceptor = acceptor.clone();

                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = TokioIo::new(tls_stream);
                        let service = hyper::service::service_fn(handle_request);
                        if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new()).serve_connection(io, service).await {
                            eprintln!("TLS HTTP/2 error: {err}");
                        }
                    }
                    Err(e) => eprintln!("TLS accept error: {e}"),
                }
            }

            #[allow(unreachable_code)]
            Ok::<(), Box<dyn std::error::Error>>(())
        } => {},
        _ = tokio::signal::ctrl_c() => {
            eprintln!("Server shutting down.");
        }
    }

    Ok(())
}
