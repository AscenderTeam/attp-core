import asyncio
import time
from statistics import mean, stdev

from python.attp_core.rs_api import AttpClientSession, Limits, PyAttpMessage, AttpCommand

TOTAL_CONNECTIONS = 20_000
BATCH_SIZE = 1000


async def connect_and_ping(i: int, latencies: list[float]):
    """Open a client session, send one message, wait for echo, record latency."""
    transport = AttpClientSession("attp://localhost:8888")
    start = time.perf_counter()
    try:
        transport = await transport.connect(10, Limits(2000))
        if not transport.session:
            return False

        # Send a message
        msg = PyAttpMessage(
            route_id=1,
            command_type=AttpCommand.EMIT,
            correlation_id=None,
            payload=f"hello-{i}".encode(),
            version=b"01",
        )
        await transport.session.send(msg)

        # Wait briefly to simulate round-trip (server must respond)
        # In a real test, you would add an event handler instead of sleep
        await asyncio.sleep(0.01)

        latencies.append(time.perf_counter() - start)
        return True
    except Exception as e:
        print(f"[{i}] failed: {e}")
        return False


async def benchmark():
    latencies: list[float] = []
    successes = 0
    start_time = time.perf_counter()

    for start in range(0, TOTAL_CONNECTIONS, BATCH_SIZE):
        tasks = [connect_and_ping(i, latencies) for i in range(start, start + BATCH_SIZE)]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        successes += sum(1 for r in results if r is True)
        print(f"Batch {start}-{start+BATCH_SIZE}: {successes} total successes so far")

    total_time = time.perf_counter() - start_time

    # Benchmark report
    print("\n===== Benchmark Results =====")
    print(f"Connections attempted: {TOTAL_CONNECTIONS}")
    print(f"Successful: {successes}")
    print(f"Failed: {TOTAL_CONNECTIONS - successes}")
    print(f"Total time: {total_time:.2f} sec")
    if latencies:
        print(f"Mean latency: {mean(latencies)*1000:.2f} ms")
        if len(latencies) > 1:
            print(f"Latency stdev: {stdev(latencies)*1000:.2f} ms")


if __name__ == "__main__":
    asyncio.run(benchmark())
