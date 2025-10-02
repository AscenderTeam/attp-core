from dataclasses import dataclass
from typing import Any, Awaitable, Callable, Literal, Optional, Self
from enum import Enum


@dataclass(frozen=False)
class Limits:
    max_payload_size: int


class AttpCommand(Enum):
    CALL = 0
    ACK = 1
    EMIT = 2
    ERR = 3

    AUTH = 4
    READY = 5
    PING = 6
    PONG = 7
    DISCONNECT = 8
    DEFER = 9


class AttpMessage:
    def __init__(
        self,
        route_id: int, 
        command_type: int, 
        correlation_id: Optional[bytes], 
        payload: Optional[bytes],
        version: bytes
    ) -> None: ...
    
    @staticmethod
    def get_from_bytes(buf: bytes, limits: Limits) -> AttpMessage: ...
    def convert_to_bytes(self) -> bytes: ...

class PyAttpMessage:
    route_id: int
    command_type: AttpCommand
    correlation_id: Optional[bytes]
    payload: Optional[bytes]
    version: bytes
    
    def __init__(self, route_id: int, command_type: AttpCommand, correlation_id: Optional[bytes], payload: Optional[bytes], version: bytes) -> None: ...
    @property
    def bytes(self) -> bytes: ...

class Session:
    session_id: Optional[str]
    peername: Optional[str]
    
    def add_event_handler(self, callback: Callable[[list[PyAttpMessage]], Any | Awaitable]) -> None: ...
    async def send(self, frame: PyAttpMessage) -> None: ...
    async def send_batch(self, frames: list[PyAttpMessage]) -> None: ...
    async def start_listener(self) -> None: ...
    async def start_handler(self) -> None: ...
    def stop_listener(self) -> None: ...
    def disconnect(self) -> None: ...


class AttpTransport:
    host: str
    port: int
    limits: Any
    
    def __init__(self, host: str, port: int, on_connection: Callable[[Session], Any], limits: Limits) -> None: ...
    async def start_server(self) -> None: 
        """
        WARN: This may block your coroutine, so use it wisely, wrap it with `asyncio.create_task(...)`
        
        As soon as you close your application, call `stop_server()` method
        """
        ...
    async def stop_server(self) -> None: 
        """
        Stops the server from accepting new connections.

        IMPORTANT:
        This does NOT automatically close active Session.
        You must explicitly call `close()` (or your own cleanup logic) on every active Session
        before stopping the server, otherwise connections may remain open.
        """
        ...


class AttpClientSession:
    """
    Client Session of ATTP protocol.
    Use `await AttpClientSession.connect(...)` to establish a connection to the ATTP server.
    Use `AttpClientSession.disconnect()` to close the connection.
    
    Use `AttpClientSession.session` to access the underlying Session object once connected.
    """
    session: Optional[Session]
    host: str
    port: int
    
    def __init__(self, connection_string: str) -> None: ...
    @property
    def is_connected(self) -> bool: ...
    async def connect(self, max_retries: int, limits: Limits) -> Self: ...
    def disconnect(self) -> None: ...