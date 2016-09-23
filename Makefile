TEX = lualatex -shell-escape -interaction=nonstopmode -file-line-error

PRE := $(wildcard ./*.tex)
OBJ := $(PRE:%.tex=%.pdf)

all: $(OBJ)

%.pdf: %.tex
	$(TEX) $(@:.pdf=.tex)

clean:
	$(RM) *.aux *.log *.out

cleanall: clean
	$(RM) *.pdf
