import asyncio

from python.attp_core.rs_api import AttpCommand, AttpTransport, Limits, PyAttpMessage, Session


async def handle_session(session: Session) -> None:
    async def on_event(messages: list[PyAttpMessage]) -> None:
        for msg in messages:
            payload = msg.payload or b""
            print(f"[server] recv from {session.session_id}: {payload!r}")
            await session.send(
                PyAttpMessage(
                    route_id=msg.route_id,
                    command_type=AttpCommand.EMIT,
                    correlation_id=msg.correlation_id,
                    payload=b"ack:" + payload if payload else b"ack",
                    version=msg.version,
                )
            )

    session.add_event_handler(on_event)
    await asyncio.gather(session.start_handler(), session.start_listener())


async def on_connection(session: Session) -> None:
    print(f"[server] connected peer={session.peername} id={session.session_id}")
    await session.send(
        PyAttpMessage(
            route_id=1,
            command_type=AttpCommand.EMIT,
            correlation_id=None,
            payload=b"hello from server",
            version=b"01",
        )
    )
    await handle_session(session)


async def main() -> None:
    transport = AttpTransport(
        "0.0.0.0",
        8888,
        on_connection=on_connection,
        limits=Limits(2000),
    )
    await transport.start_server()


if __name__ == "__main__":
    asyncio.run(main())
