"""Finds the final renders and models a run left in its slot."""
import os


def models(folder):
    """Returns the stems of the folder's models, each named by a report."""
    return sorted(n[: -len("-report.txt")] for n in os.listdir(folder) if n.endswith("-report.txt"))


def is_sheet(name, stems):
    return name.endswith(".png") and name[:-4].removesuffix("-review") in stems


def find_sheet(folder):
    """Returns the folder's first review sheet, the PNG named for a report's
    model with or without `-review`, or None."""
    stems = set(models(folder))
    return next((n for n in sorted(os.listdir(folder)) if is_sheet(n, stems)), None)


def other_renders(folder, sheet):
    return [n for n in sorted(os.listdir(folder)) if n.endswith(".png") and n != sheet]


def extra_renders(folder):
    stems = set(models(folder))
    names = [n for n in os.listdir(folder) if n.endswith(".png") and not is_sheet(n, stems)]
    return sorted(names, key=lambda n: os.path.getmtime(os.path.join(folder, n)), reverse=True)
