use std::collections::HashMap;

use global::shutdown_tracer_provider;
use opentelemetry::{
    global,
    propagation::TextMapPropagator,
    sdk::{export::trace::stdout, propagation::TraceContextPropagator},
    Context,
};
use tracing::{info_span, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{prelude::*, Registry};

fn with_subscriber(f: impl FnOnce()) {
    let tracer = stdout::new_pipeline().install_simple();

    tracing::subscriber::with_default(
        Registry::default().with(tracing_opentelemetry::layer().with_tracer(tracer)),
        f,
    );

    shutdown_tracer_provider(); // sending remaining spans
}

fn get_traceparent(span: &Span) -> String {
    let cx = span.context();

    let propagator = TraceContextPropagator::new();

    let mut injector = HashMap::new();

    propagator.inject_context(&cx, &mut injector);

    injector.remove("traceparent").unwrap()
}

fn main() {
    with_subscriber(|| {
        let traceparent = {
            let span = info_span!("foo");

            get_traceparent(&span)
        };

        {
            let span = info_span!("bar");

            let propagator = TraceContextPropagator::new();
            let cx = Context::new();

            let mut extractor = HashMap::new();
            extractor.insert(String::from("traceparent"), traceparent.clone());

            propagator.extract_with_context(&cx, &extractor);

            span.set_parent(cx);

            assert_eq!(get_traceparent(&span), traceparent);
        }
    });
}
