" ===============================================
"               .VIMRC 
"
" Stolen bits and there from across the interwebs.
" Mainly from:
"   - andrewrk
" ===============================================

" ===============================================
" EDITOR SETTINGS
" ===============================================

" SPACES AND TABS
set tabstop=4                       " number of visual spaces per TAB
set softtabstop=4                   " number of spaces in tab when editing
set expandtab                       " tabs are spaces
set autoindent                      " always turn on indentation
set breakindent                     " Wrap lines at same indent level
set shiftwidth=4                    " Spaces to use for autoindenting
set backspace=indent,eol,start      " proper backspace behavior
set textwidth=110                   " set textwidth to be 110

"UI CONFIG
set number 	                        " show line numbers
set relativenumber                  " show the relative line number
set showcmd                         " show command in bottom bar
set cursorline                      " highlight current line
set wildmenu                        " visual autocomplete for command menu
set wildmode=longest,full           " Enable file autocomplete in command mode
set lazyredraw                      " redraw only when we need to.
set showmatch                       " highlight matching [{()}]
set scrolloff=20                    " always have x lines below

"SEARCHING
set incsearch                       " search as characters are entered
set hlsearch                        " highlight matches

" Wrapping options
" set formatoptions=tc                " wrap text and comments using textwidth
set formatoptions+=r                " continue comments when pressing ENTER in I mode
set formatoptions+=q                " enable formatting of comments with gq
set formatoptions+=n                " detect lists for formatting
" set formatoptions+=b                " auto-wrap in insert mode, and do not wrap old long lines

" add yaml stuffs
au! BufNewFile,BufReadPost *.{yaml,yml} set filetype=yaml foldmethod=indent
autocmd FileType yaml setlocal ts=2 sts=2 sw=2 expandtab

" lets you click around
set mouse=a

" lets you resize the buffer with mouse
set ttymouse=xterm2

" the bar at the character limit
set colorcolumn=110

" don't let the swap file all around the system
set directory^=$HOME/.vim/swap/

" so we can run cargo fmt in peace
" set noswapfile

" ===========================================
" CUSTOM KEYBINDINGS
" ===========================================

" Set space as the leader
let mapleader = "\<Space>"

" Quick-save
nmap <leader>w :w<CR>

" <leader><leader> toggles between buffers
nnoremap <leader><leader> <c-^>

" jk is escape
inoremap jk <esc>

" No arrow keys --- force yourself to use the home row
nnoremap <up> <nop>
nnoremap <down> <nop>
inoremap <up> <nop>
inoremap <down> <nop>
inoremap <left> <nop>
inoremap <right> <nop>

" Left and right can switch buffers
nnoremap <left> :bp<CR>
nnoremap <right> :bn<CR>

" from https://linuxhint.com/vim_split_screen/
" remap keys to change windows when splitting vim
nnoremap <C-J> <C-W><C-J>
nnoremap <C-K> <C-W><C-K>
nnoremap <C-L> <C-W><C-L>
nnoremap <C-H> <C-W><C-H>

"nnoremap <C--> <C-W><C-S>
"nnoremap <C-|> <C-W><C-V>
nnoremap <C-S> <C-W><C-S>
nnoremap <C-V> <C-W><C-V>

" Toggle paste mode on and off
map <leader>v :setlocal paste!<cr>

" nice try, Ex mode
map Q <Nop>

" ===========================================
" PLUGINS
" ===========================================
set rtp+=/opt/homebrew/opt/fzf

call plug#begin('~/.vim/plugged')
    Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }
    Plug 'junegunn/fzf.vim'
    Plug 'lotabout/skim', { 'dir': '~/.skim', 'do': './install' }
    Plug 'lotabout/skim.vim'
call plug#end()

" File finder
nmap <Leader>t :FZF<CR>
nmap <Leader>h :Rg<CR>
