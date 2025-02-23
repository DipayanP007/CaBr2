import { Provider, ProviderMapping, SearchArguments, SearchResult, SearchType } from './provider.model';

import { BehaviorSubject, Observable } from 'rxjs';
import { SubstanceData } from '../../models/substances.model';

export abstract class IProviderService {
  abstract providerMappingsSubject: BehaviorSubject<ProviderMapping>;

  abstract providerMappingsObservable: Observable<ProviderMapping>;

  /**
   * Returns a Provider[] with the names and identifiers of the available Providers.
   *
   * For example:
   *
   * ```ts
   * [
   *   {
   *     name: 'Gestis',
   *     identifier: 'gestis',
   *   }
   * ]
   * ```
   */
  abstract getAvailableProviders(): Observable<Provider[]>;

  /**
   * Returns a string[] with names to use in an search query.
   *
   * For example:
   *
   * ```ts
   * [
   *   'wasser',
   *   'wasserstoff',
   *   'wasserstoffperoxid'
   * ]
   * ```
   */
  abstract searchSuggestions(provider: string, searchType: SearchType, query: string): Observable<string[]>;

  /**
   * Returns a SearchResult[] with objects!.
   *
   * For example:
   *
   * ```ts
   * // TODO
   * ```
   *
   * returns:
   *
   * ```ts
   * [
   *   {name: 'Wasser', casNumber: '7732-18-5', zvgNumber: '001140'},
   *   {name: 'Wasserstoff', casNumber: '1333-74-0', zvgNumber: '007010'},
   *   {name: 'wasserstoffperoxid', casNumber: '7722-84-1', zvgNumber: '536373'}
   * ]
   * ```
   */
  abstract search(provider: string, args: SearchArguments): Observable<SearchResult[]>;

  /**
   * Returns the parsed data of a substance from the given provider or an error
   * stating the cause of the failure when parsing the data.
   */
  abstract substanceData(provider: string, identifier: string): Observable<SubstanceData>;
}
