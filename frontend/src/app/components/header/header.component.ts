import { Component, OnInit } from '@angular/core';

import { GlobalModel } from '../../@core/models/global.model';
import { Header } from '../../@core/interfaces/Header';

@Component({
  selector: 'app-header',
  templateUrl: './header.component.html',
  styleUrls: ['./header.component.scss'],
})
export class HeaderComponent implements OnInit {
  header!: Header;

  constructor(public globals: GlobalModel) {}

  ngOnInit(): void {
    this.globals.headerObservable.subscribe((data) => (this.header = data));
  }
}
