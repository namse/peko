use deno_core::anyhow::Result;
use deno_core::{JsRuntime, RuntimeOptions, extension, op2, resolve_url, v8::CreateParams};

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let code = include_str!("../example-hello-world/dist/bundle.js");
            run(code).await?;
            Ok::<(), deno_core::anyhow::Error>(())
        })?;
    Ok(())
}

extension!(
    bootstrap,
    esm_entry_point = "ext:bootstrap/bootstrap.js",
    esm = ["bootstrap.js"],
);

#[op2(fast)]
pub fn op_custom_print(#[string] msg: &str) {
    println!("[MyRust-Log] {msg}");
}

fn create_runtime() -> JsRuntime {
    let create_params = CreateParams::default();

    let runtime_options = RuntimeOptions {
        extensions: vec![
            deno_webidl::deno_webidl::init(),
            deno_web::deno_web::init(Default::default()),
            deno_fetch::deno_fetch::init(Default::default()),
            bootstrap::init(),
        ],
        create_params: Some(create_params),
        ..Default::default()
    };

    JsRuntime::new(runtime_options)
}

async fn run(script: &str) -> Result<()> {
    let mut runtime = create_runtime();

    let mod_id = runtime
        .load_main_es_module_from_code(&resolve_url("fn0://main.js").unwrap(), script.to_string())
        .await?;
    let result = runtime.mod_evaluate(mod_id);
    runtime.run_event_loop(Default::default()).await?;
    result.await?;

    let namespace_global = runtime.get_module_namespace(mod_id)?;

    let result_promise_global = {
        deno_core::scope!(scope, runtime);
        let namespace = deno_core::v8::Local::<deno_core::v8::Object>::new(scope, namespace_global);

        let default_key = deno_core::v8::String::new(scope, "default").unwrap();
        let default_export = namespace.get(scope, default_key.into()).unwrap();

        let func = deno_core::v8::Local::<deno_core::v8::Function>::try_from(default_export)?;

        let req_script =
            "new Request('http://localhost', { method: 'POST', body: 'Hello from Rust!' })";
        let setup_script_source = deno_core::v8::String::new(scope, req_script).unwrap();
        let setup_script =
            deno_core::v8::Script::compile(scope, setup_script_source, None).unwrap();
        let arg_val = setup_script.run(scope).unwrap();

        let recv = deno_core::v8::undefined(scope).into();
        let result_promise_val = func.call(scope, recv, &[arg_val]).unwrap();

        deno_core::v8::Global::new(scope, result_promise_val)
    };

    let result_global = {
        let result = runtime.resolve(result_promise_global);
        runtime.run_event_loop(Default::default()).await?;
        result.await?
    };

    deno_core::scope!(scope, runtime);

    let result_val = deno_core::v8::Local::<deno_core::v8::Value>::new(scope, result_global);

    if result_val.is_object() {
        let stringified = result_val.to_detail_string(scope).unwrap();
        let rust_string = stringified.to_rust_string_lossy(scope);
        println!("JS Function Result: {}", rust_string);
    }

    Ok(())
}
