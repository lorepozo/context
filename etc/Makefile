TEX = xelatex -shell-escape -interaction=nonstopmode -file-line-error

PRE := $(wildcard ./*.tex)
OBJ := $(PRE:%.tex=%.pdf)

all: $(OBJ)

%.pdf: %.tex
	$(TEX) $(@:.pdf=.tex)

post: paper.pdf
	scp paper.pdf me@lucasem.com:docs/context.pdf

clean:
	$(RM) *.aux *.log *.out

cleanall: clean
	$(RM) *.pdf
