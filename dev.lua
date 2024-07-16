return function(x)
    flakes.require({
        nixpkg("rustc"), nixpkg("cargo"), cratesio("bacon")
    })
end