import asyncio
from python.attp_core.rs_api import AttpClientSession, AttpCommand, Limits, PyAttpMessage, Session, init_logging


async def main():
    transport = AttpClientSession("attp://localhost:8888", Limits(2000))
    init_logging()
    
    transport = await transport.connect(10)
    
    async def on_event(msg: list[PyAttpMessage]):
        print(type(msg))
        print("Received message from server:", msg[0].payload)
        
        assert transport.session
        
        await transport.session.send(PyAttpMessage(route_id=3, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Received", version=b'01'))
    
    if not transport.session:
        print("Failed to connect to server")
        return
    
    await transport.session.send(PyAttpMessage(route_id=1, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Hello, server!", version=b'01'))
    transport.session.add_event_handler(on_event)
    
    await asyncio.gather(
        transport.session.start_handler(),
        transport.session.start_listener()
    )


asyncio.run(main())
