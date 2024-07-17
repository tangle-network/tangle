/// Message type to push new wasm code
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPushNewWasmCode {
	#[prost(string, tag = "1")]
	pub signer: ::prost::alloc::string::String,
	#[prost(bytes = "vec", tag = "3")]
	pub code: ::prost::alloc::vec::Vec<u8>,
}
/// Response in case of successful handling
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPushNewWasmCodeResponse {
	#[prost(bytes = "vec", tag = "1")]
	pub code_id: ::prost::alloc::vec::Vec<u8>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
	#![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
	use tonic::codegen::{http::Uri, *};
	/// Msg defines the ibc/wasm Msg service.
	#[derive(Debug, Clone)]
	pub struct MsgClient<T> {
		inner: tonic::client::Grpc<T>,
	}
	impl MsgClient<tonic::transport::Channel> {
		/// Attempt to create a new client by connecting to a given endpoint.
		pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
		where
			D: std::convert::TryInto<tonic::transport::Endpoint>,
			D::Error: Into<StdError>,
		{
			let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
			Ok(Self::new(conn))
		}
	}
	impl<T> MsgClient<T>
	where
		T: tonic::client::GrpcService<tonic::body::BoxBody>,
		T::Error: Into<StdError>,
		T::ResponseBody: Body<Data = Bytes> + Send + 'static,
		<T::ResponseBody as Body>::Error: Into<StdError> + Send,
	{
		pub fn new(inner: T) -> Self {
			let inner = tonic::client::Grpc::new(inner);
			Self { inner }
		}
		pub fn with_origin(inner: T, origin: Uri) -> Self {
			let inner = tonic::client::Grpc::with_origin(inner, origin);
			Self { inner }
		}
		pub fn with_interceptor<F>(inner: T, interceptor: F) -> MsgClient<InterceptedService<T, F>>
		where
			F: tonic::service::Interceptor,
			T::ResponseBody: Default,
			T: tonic::codegen::Service<
				http::Request<tonic::body::BoxBody>,
				Response = http::Response<
					<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
				>,
			>,
			<T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
				Into<StdError> + Send + Sync,
		{
			MsgClient::new(InterceptedService::new(inner, interceptor))
		}
		/// Compress requests with the given encoding.
		///
		/// This requires the server to support it otherwise it might respond with an
		/// error.
		#[must_use]
		pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.inner = self.inner.send_compressed(encoding);
			self
		}
		/// Enable decompressing responses.
		#[must_use]
		pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.inner = self.inner.accept_compressed(encoding);
			self
		}
		/// PushNewWasmCode defines a rpc handler method for PushNewWasmCode.
		pub async fn push_new_wasm_code(
			&mut self,
			request: impl tonic::IntoRequest<super::MsgPushNewWasmCode>,
		) -> Result<tonic::Response<super::MsgPushNewWasmCodeResponse>, tonic::Status> {
			self.inner.ready().await.map_err(|e| {
				tonic::Status::new(
					tonic::Code::Unknown,
					format!("Service was not ready: {}", e.into()),
				)
			})?;
			let codec = tonic::codec::ProstCodec::default();
			let path = http::uri::PathAndQuery::from_static(
				"/ibc.lightclients.wasm.v1.Msg/PushNewWasmCode",
			);
			self.inner.unary(request.into_request(), path, codec).await
		}
	}
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod msg_server {
	#![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
	use tonic::codegen::*;
	/// Generated trait containing gRPC methods that should be implemented for use with MsgServer.
	#[async_trait]
	pub trait Msg: Send + Sync + 'static {
		/// PushNewWasmCode defines a rpc handler method for PushNewWasmCode.
		async fn push_new_wasm_code(
			&self,
			request: tonic::Request<super::MsgPushNewWasmCode>,
		) -> Result<tonic::Response<super::MsgPushNewWasmCodeResponse>, tonic::Status>;
	}
	/// Msg defines the ibc/wasm Msg service.
	#[derive(Debug)]
	pub struct MsgServer<T: Msg> {
		inner: _Inner<T>,
		accept_compression_encodings: EnabledCompressionEncodings,
		send_compression_encodings: EnabledCompressionEncodings,
	}
	struct _Inner<T>(Arc<T>);
	impl<T: Msg> MsgServer<T> {
		pub fn new(inner: T) -> Self {
			Self::from_arc(Arc::new(inner))
		}
		pub fn from_arc(inner: Arc<T>) -> Self {
			let inner = _Inner(inner);
			Self {
				inner,
				accept_compression_encodings: Default::default(),
				send_compression_encodings: Default::default(),
			}
		}
		pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
		where
			F: tonic::service::Interceptor,
		{
			InterceptedService::new(Self::new(inner), interceptor)
		}
		/// Enable decompressing requests with the given encoding.
		#[must_use]
		pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.accept_compression_encodings.enable(encoding);
			self
		}
		/// Compress responses with the given encoding, if the client supports it.
		#[must_use]
		pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.send_compression_encodings.enable(encoding);
			self
		}
	}
	impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
	where
		T: Msg,
		B: Body + Send + 'static,
		B::Error: Into<StdError> + Send + 'static,
	{
		type Response = http::Response<tonic::body::BoxBody>;
		type Error = std::convert::Infallible;
		type Future = BoxFuture<Self::Response, Self::Error>;
		fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
			Poll::Ready(Ok(()))
		}
		fn call(&mut self, req: http::Request<B>) -> Self::Future {
			let inner = self.inner.clone();
			match req.uri().path() {
				"/ibc.lightclients.wasm.v1.Msg/PushNewWasmCode" => {
					#[allow(non_camel_case_types)]
					struct PushNewWasmCodeSvc<T: Msg>(pub Arc<T>);
					impl<T: Msg> tonic::server::UnaryService<super::MsgPushNewWasmCode> for PushNewWasmCodeSvc<T> {
						type Response = super::MsgPushNewWasmCodeResponse;
						type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
						fn call(
							&mut self,
							request: tonic::Request<super::MsgPushNewWasmCode>,
						) -> Self::Future {
							let inner = self.0.clone();
							let fut = async move { (*inner).push_new_wasm_code(request).await };
							Box::pin(fut)
						}
					}
					let accept_compression_encodings = self.accept_compression_encodings;
					let send_compression_encodings = self.send_compression_encodings;
					let inner = self.inner.clone();
					let fut = async move {
						let inner = inner.0;
						let method = PushNewWasmCodeSvc(inner);
						let codec = tonic::codec::ProstCodec::default();
						let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
							accept_compression_encodings,
							send_compression_encodings,
						);
						let res = grpc.unary(method, req).await;
						Ok(res)
					};
					Box::pin(fut)
				},
				_ => Box::pin(async move {
					Ok(http::Response::builder()
						.status(200)
						.header("grpc-status", "12")
						.header("content-type", "application/grpc")
						.body(empty_body())
						.unwrap())
				}),
			}
		}
	}
	impl<T: Msg> Clone for MsgServer<T> {
		fn clone(&self) -> Self {
			let inner = self.inner.clone();
			Self {
				inner,
				accept_compression_encodings: self.accept_compression_encodings,
				send_compression_encodings: self.send_compression_encodings,
			}
		}
	}
	impl<T: Msg> Clone for _Inner<T> {
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}
	impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{:?}", self.0)
		}
	}
	impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
		const NAME: &'static str = "ibc.lightclients.wasm.v1.Msg";
	}
}
/// WasmCode query
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WasmCodeQuery {
	#[prost(string, tag = "1")]
	pub code_id: ::prost::alloc::string::String,
}
/// WasmCode response
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WasmCodeResponse {
	#[prost(bytes = "vec", tag = "1")]
	pub code: ::prost::alloc::vec::Vec<u8>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
	#![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
	use tonic::codegen::{http::Uri, *};
	/// Query service for wasm module
	#[derive(Debug, Clone)]
	pub struct QueryClient<T> {
		inner: tonic::client::Grpc<T>,
	}
	impl QueryClient<tonic::transport::Channel> {
		/// Attempt to create a new client by connecting to a given endpoint.
		pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
		where
			D: std::convert::TryInto<tonic::transport::Endpoint>,
			D::Error: Into<StdError>,
		{
			let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
			Ok(Self::new(conn))
		}
	}
	impl<T> QueryClient<T>
	where
		T: tonic::client::GrpcService<tonic::body::BoxBody>,
		T::Error: Into<StdError>,
		T::ResponseBody: Body<Data = Bytes> + Send + 'static,
		<T::ResponseBody as Body>::Error: Into<StdError> + Send,
	{
		pub fn new(inner: T) -> Self {
			let inner = tonic::client::Grpc::new(inner);
			Self { inner }
		}
		pub fn with_origin(inner: T, origin: Uri) -> Self {
			let inner = tonic::client::Grpc::with_origin(inner, origin);
			Self { inner }
		}
		pub fn with_interceptor<F>(
			inner: T,
			interceptor: F,
		) -> QueryClient<InterceptedService<T, F>>
		where
			F: tonic::service::Interceptor,
			T::ResponseBody: Default,
			T: tonic::codegen::Service<
				http::Request<tonic::body::BoxBody>,
				Response = http::Response<
					<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
				>,
			>,
			<T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
				Into<StdError> + Send + Sync,
		{
			QueryClient::new(InterceptedService::new(inner, interceptor))
		}
		/// Compress requests with the given encoding.
		///
		/// This requires the server to support it otherwise it might respond with an
		/// error.
		#[must_use]
		pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.inner = self.inner.send_compressed(encoding);
			self
		}
		/// Enable decompressing responses.
		#[must_use]
		pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.inner = self.inner.accept_compressed(encoding);
			self
		}
		/// Get Wasm code for given code id
		pub async fn wasm_code(
			&mut self,
			request: impl tonic::IntoRequest<super::WasmCodeQuery>,
		) -> Result<tonic::Response<super::WasmCodeResponse>, tonic::Status> {
			self.inner.ready().await.map_err(|e| {
				tonic::Status::new(
					tonic::Code::Unknown,
					format!("Service was not ready: {}", e.into()),
				)
			})?;
			let codec = tonic::codec::ProstCodec::default();
			let path =
				http::uri::PathAndQuery::from_static("/ibc.lightclients.wasm.v1.Query/WasmCode");
			self.inner.unary(request.into_request(), path, codec).await
		}
	}
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod query_server {
	#![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
	use tonic::codegen::*;
	/// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
	#[async_trait]
	pub trait Query: Send + Sync + 'static {
		/// Get Wasm code for given code id
		async fn wasm_code(
			&self,
			request: tonic::Request<super::WasmCodeQuery>,
		) -> Result<tonic::Response<super::WasmCodeResponse>, tonic::Status>;
	}
	/// Query service for wasm module
	#[derive(Debug)]
	pub struct QueryServer<T: Query> {
		inner: _Inner<T>,
		accept_compression_encodings: EnabledCompressionEncodings,
		send_compression_encodings: EnabledCompressionEncodings,
	}
	struct _Inner<T>(Arc<T>);
	impl<T: Query> QueryServer<T> {
		pub fn new(inner: T) -> Self {
			Self::from_arc(Arc::new(inner))
		}
		pub fn from_arc(inner: Arc<T>) -> Self {
			let inner = _Inner(inner);
			Self {
				inner,
				accept_compression_encodings: Default::default(),
				send_compression_encodings: Default::default(),
			}
		}
		pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
		where
			F: tonic::service::Interceptor,
		{
			InterceptedService::new(Self::new(inner), interceptor)
		}
		/// Enable decompressing requests with the given encoding.
		#[must_use]
		pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.accept_compression_encodings.enable(encoding);
			self
		}
		/// Compress responses with the given encoding, if the client supports it.
		#[must_use]
		pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
			self.send_compression_encodings.enable(encoding);
			self
		}
	}
	impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
	where
		T: Query,
		B: Body + Send + 'static,
		B::Error: Into<StdError> + Send + 'static,
	{
		type Response = http::Response<tonic::body::BoxBody>;
		type Error = std::convert::Infallible;
		type Future = BoxFuture<Self::Response, Self::Error>;
		fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
			Poll::Ready(Ok(()))
		}
		fn call(&mut self, req: http::Request<B>) -> Self::Future {
			let inner = self.inner.clone();
			match req.uri().path() {
				"/ibc.lightclients.wasm.v1.Query/WasmCode" => {
					#[allow(non_camel_case_types)]
					struct WasmCodeSvc<T: Query>(pub Arc<T>);
					impl<T: Query> tonic::server::UnaryService<super::WasmCodeQuery> for WasmCodeSvc<T> {
						type Response = super::WasmCodeResponse;
						type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
						fn call(
							&mut self,
							request: tonic::Request<super::WasmCodeQuery>,
						) -> Self::Future {
							let inner = self.0.clone();
							let fut = async move { (*inner).wasm_code(request).await };
							Box::pin(fut)
						}
					}
					let accept_compression_encodings = self.accept_compression_encodings;
					let send_compression_encodings = self.send_compression_encodings;
					let inner = self.inner.clone();
					let fut = async move {
						let inner = inner.0;
						let method = WasmCodeSvc(inner);
						let codec = tonic::codec::ProstCodec::default();
						let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
							accept_compression_encodings,
							send_compression_encodings,
						);
						let res = grpc.unary(method, req).await;
						Ok(res)
					};
					Box::pin(fut)
				},
				_ => Box::pin(async move {
					Ok(http::Response::builder()
						.status(200)
						.header("grpc-status", "12")
						.header("content-type", "application/grpc")
						.body(empty_body())
						.unwrap())
				}),
			}
		}
	}
	impl<T: Query> Clone for QueryServer<T> {
		fn clone(&self) -> Self {
			let inner = self.inner.clone();
			Self {
				inner,
				accept_compression_encodings: self.accept_compression_encodings,
				send_compression_encodings: self.send_compression_encodings,
			}
		}
	}
	impl<T: Query> Clone for _Inner<T> {
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}
	impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{:?}", self.0)
		}
	}
	impl<T: Query> tonic::server::NamedService for QueryServer<T> {
		const NAME: &'static str = "ibc.lightclients.wasm.v1.Query";
	}
}
/// Wasm light client's Client state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
	#[prost(bytes = "vec", tag = "1")]
	pub data: ::prost::alloc::vec::Vec<u8>,
	#[prost(bytes = "vec", tag = "2")]
	pub code_id: ::prost::alloc::vec::Vec<u8>,
	#[prost(message, optional, tag = "3")]
	pub latest_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
/// Wasm light client's ConsensusState
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
	#[prost(bytes = "vec", tag = "1")]
	pub data: ::prost::alloc::vec::Vec<u8>,
	/// timestamp that corresponds to the block height in which the ConsensusState
	/// was stored.
	#[prost(uint64, tag = "2")]
	pub timestamp: u64,
}
/// Wasm light client Header
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
	#[prost(bytes = "vec", tag = "1")]
	pub data: ::prost::alloc::vec::Vec<u8>,
	#[prost(message, optional, tag = "2")]
	pub height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
/// Wasm light client Misbehaviour
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
	#[prost(bytes = "vec", tag = "1")]
	pub data: ::prost::alloc::vec::Vec<u8>,
}
