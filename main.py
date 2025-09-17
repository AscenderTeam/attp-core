import asyncio
from python.attp_core.rs_api import AttpCommand, AttpTransport, Limits, PyAttpMessage, Session


async def run_listener(session: Session):
    await asyncio.gather(session.start_handler(), session.start_listener())

    
async def main():
    async def on_connection(session: Session):
        async def on_event(messages: list[PyAttpMessage]):
            try:
                print("PYTHON HANDLER GOT:", messages[0].command_type)
                print(type(messages[0].command_type))
                if messages[0].command_type == AttpCommand.PING:
                    print("PYTHON HANDLER IS SENDING PONG!")
                    await session.send(PyAttpMessage(route_id=messages[0].route_id, command_type=AttpCommand.PONG, correlation_id=None, payload=None, version=b'01'))

            except Exception as e:
                print("Error in event handler:", e)
            
        session.add_event_handler(on_event)
        asyncio.create_task(run_listener(session))
        print("New session:", session.session_id)
        # await session.start_listener()
        await session.send(PyAttpMessage(route_id=1, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Hello, client!", version=b'01'))
        # await session.send(PyAttpMessage(route_id=1, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Hello, client!", version=b'01'))
        # await session.send(PyAttpMessage(route_id=1, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Hello, client!", version=b'01'))
        # await session.send(PyAttpMessage(route_id=1, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Hello, client!", version=b'01'))
        await asyncio.sleep(5)
        await session.send(PyAttpMessage(route_id=1, command_type=AttpCommand.EMIT, correlation_id=None, payload=b"Hello again, client!", version=b'01'))

        
    transport = AttpTransport("0.0.0.0", 8888, on_connection=on_connection, limits=Limits(2000))
    
    await transport.start_server()


asyncio.run(main())