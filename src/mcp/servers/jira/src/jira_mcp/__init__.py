from logging import INFO, getLogger, Formatter, StreamHandler

logger = getLogger(__name__)
handler = StreamHandler()
handler.setFormatter(
    Formatter("%(asctime)s - [%(name)s] - %(levelname)s - %(message)s")
)
logger.addHandler(handler)
logger.setLevel(INFO)
