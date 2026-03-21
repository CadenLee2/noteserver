# Example File

[Noteserver](https://github.com/Cadecraft/noteserver) is a tool for self-hosting markdown notes. These notes can be divided into folders, so you can have a structure like this:
```
recipes/
    pasta
    soup
tutorials/
    learn-markdown
    rust
```
An end user can visit `/recipes/soup` to see your soup recipe!

## Directory protection
Let's say your family has passed down some secret recipes that they want to keep hidden, but for the holidays everyone wants to help cook.

You can *protect* your `secret_recipes` directory, then share a magic link to grant them access:
```
/secret_recipes?tok=somesecrettoken
```

## Admin tools
Run [scripts/noteadmin.py](https://github.com/Cadecraft/noteserver/blob/main/scripts/noteadmin.py) from any directory! If you use it often, consider adding it to your path.

## More details
### Rendering
GFM ([GitHub Flavored Markdown](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github)) is supported, so you can include tables, ~~strikethroughs~~, and more!
