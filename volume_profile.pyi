from typing import Sized


def compute_volume_profile(close: Sized, volume: Sized, bins: int, window: int
                   ) -> tuple[list[float], list[dict[int, float]]]:
    """Computes volume profile.

    Parameters
    ----------
    close : Iterable
        Close price.
    volume : Iterable
        Trading Volume.
    bins : int
        Number of bins that close prices that will be binned into.
    window : int
        Look back period.

    Returns
    -------
    tuple[list[float], list[dict[int, float]]]
        Point of control and Volume profile.
    """